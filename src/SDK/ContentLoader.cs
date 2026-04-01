using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK.Dependencies;
using DINOForge.SDK.Diagnostics;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace DINOForge.SDK
{
    /// <summary>
    /// Orchestrates pack loading while delegating filesystem discovery, schema resolution,
    /// and registry registration to specialized SDK services.
    /// </summary>
    public class ContentLoader : IPackReloadService
    {
        private readonly IContentDiscoveryService _discoveryService;
        private readonly ISchemaResolverService _schemaResolver;
        private readonly IRegistryImportService _registryImport;
        private readonly PackLoader _packLoader;
        private readonly PackDependencyResolver _dependencyResolver;

        // Stored so RegisterAssetSwaps() can enumerate units/buildings after registration.
        private readonly RegistryManager? _registryManager;

        /// <summary>
        /// All stat override definitions loaded from packs.
        /// </summary>
        public IReadOnlyList<StatOverrideDefinition> LoadedOverrides => _registryImport.LoadedOverrides;

        /// <summary>
        /// Errors from the last load operation.
        /// </summary>
        public IReadOnlyList<string> LastLoadErrors { get; private set; } = new List<string>().AsReadOnly();

        /// <summary>
        /// Number of errors from the last load operation.
        /// </summary>
        public int LastLoadErrorCount => LastLoadErrors.Count;

        /// <summary>
        /// Initializes a new <see cref="ContentLoader"/> with the default SDK services.
        /// </summary>
        /// <param name="registryManager">The registry manager to populate.</param>
        /// <param name="schemaValidator">Optional schema validator. Pass null to skip validation.</param>
        /// <param name="log">Logging callback. Pass null for no logging.</param>
        public ContentLoader(RegistryManager registryManager, ISchemaValidator? schemaValidator = null, Action<string>? log = null)
            : this(
                new ContentDiscoveryService(),
                new SchemaResolverService(),
                CreateRegistryImport(registryManager, schemaValidator, log),
                registryManager)
        {
        }

        /// <summary>
        /// Initializes a new <see cref="ContentLoader"/> with custom internal services.
        /// </summary>
        /// <param name="discoveryService">Content discovery service.</param>
        /// <param name="schemaResolver">Schema resolution service.</param>
        /// <param name="registryImport">Registry import service.</param>
        /// <param name="registryManager">
        /// Optional registry manager reference used to wire <see cref="DINOForge.SDK.Assets.AssetSwapRegistry"/>
        /// after units and buildings are loaded. Pass null to skip asset swap registration.
        /// </param>
        internal ContentLoader(
            IContentDiscoveryService discoveryService,
            ISchemaResolverService schemaResolver,
            IRegistryImportService registryImport,
            RegistryManager? registryManager = null)
        {
            _discoveryService = discoveryService ?? throw new ArgumentNullException(nameof(discoveryService));
            _schemaResolver = schemaResolver ?? throw new ArgumentNullException(nameof(schemaResolver));
            _registryImport = registryImport ?? throw new ArgumentNullException(nameof(registryImport));
            _registryManager = registryManager;
            _packLoader = new PackLoader();
            _dependencyResolver = new PackDependencyResolver();
        }

        private static IRegistryImportService CreateRegistryImport(
            RegistryManager registryManager,
            ISchemaValidator? schemaValidator,
            Action<string>? log)
        {
            if (registryManager == null)
            {
                throw new ArgumentNullException(nameof(registryManager));
            }

            Action<string> logger = log ?? (_ => { });
            IDeserializer deserializer = YamlLoader.Deserializer;

            return new RegistryImportService(
                registryManager,
                schemaValidator,
                new SchemaResolverService(),
                deserializer,
                new List<StatOverrideDefinition>(),
                logger);
        }

        /// <summary>
        /// Loads a single pack from a directory containing pack.yaml.
        /// </summary>
        /// <param name="packDirectory">Path to the pack directory.</param>
        /// <returns>Result indicating success or failure with errors.</returns>
        public ContentLoadResult LoadPack(string packDirectory)
        {
            if (packDirectory == null)
            {
                throw new ArgumentNullException(nameof(packDirectory));
            }

            SentryInitializer.AddBreadcrumb($"Loading pack from: {packDirectory}", "pack");

            string manifestPath = Path.Combine(packDirectory, "pack.yaml");
            if (!File.Exists(manifestPath))
            {
                List<string> errors = new List<string> { $"Pack manifest not found: {manifestPath}" };
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            PackManifest manifest;
            try
            {
                manifest = _packLoader.LoadFromFile(manifestPath);
            }
            catch (Exception ex)
            {
                SentryInitializer.CaptureException(ex, context: "ContentLoader.LoadPack");
                List<string> errors = new List<string> { $"Failed to parse manifest at {manifestPath}: {ex.Message}" };
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            List<string> loadErrors = new List<string>();
            LoadManifestContent(packDirectory, manifest, loadErrors);

            // Wire AssetSwapRegistry for all units and buildings with visual_asset references.
            RegisterAssetSwaps(packDirectory);

            LastLoadErrors = loadErrors.AsReadOnly();
            IReadOnlyList<string> loadedPackIds = new List<string> { manifest.Id }.AsReadOnly();
            SentryInitializer.AddBreadcrumb($"Pack loaded: {manifest.Id} ({loadErrors.Count} errors)", "pack");

            return loadErrors.Count > 0
                ? ContentLoadResult.Partial(loadedPackIds, LastLoadErrors)
                : ContentLoadResult.Success(loadedPackIds);
        }

        ContentLoadResult IPackReloadService.ReloadPack(string packDirectory)
        {
            return LoadPack(packDirectory);
        }

        /// <summary>
        /// Discovers and loads all packs in a root directory, resolving dependencies.
        /// </summary>
        /// <param name="packsRootDirectory">Root directory containing pack subdirectories.</param>
        /// <returns>Aggregate result of loading all packs.</returns>
        public ContentLoadResult LoadPacks(string packsRootDirectory)
        {
            if (!Directory.Exists(packsRootDirectory))
            {
                List<string> pathErrors = new List<string> { $"Packs directory not found: {packsRootDirectory}" };
                LastLoadErrors = pathErrors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            List<string> errors = new List<string>();
            List<(string Directory, PackManifest Manifest)> manifests = DiscoverPackManifests(packsRootDirectory, errors);
            if (manifests.Count == 0)
            {
                LastLoadErrors = errors.AsReadOnly();
                return errors.Count > 0
                    ? ContentLoadResult.Failure(LastLoadErrors)
                    : ContentLoadResult.Success(new List<string>().AsReadOnly());
            }

            DependencyResult dependencyResult = _dependencyResolver.ComputeLoadOrder(manifests.Select(item => item.Manifest));
            if (!dependencyResult.IsSuccess)
            {
                errors.AddRange(dependencyResult.Errors);
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            errors.AddRange(_dependencyResolver.DetectConflicts(manifests.Select(item => item.Manifest)));

            Dictionary<string, string> directoriesByPackId = manifests.ToDictionary(
                item => item.Manifest.Id,
                item => item.Directory,
                StringComparer.OrdinalIgnoreCase);

            List<string> loadedPacks = new List<string>();
            foreach (PackManifest orderedManifest in dependencyResult.LoadOrder)
            {
                if (!directoriesByPackId.TryGetValue(orderedManifest.Id, out string? packDirectory))
                {
                    continue;
                }

                ContentLoadResult packResult = LoadPack(packDirectory);
                loadedPacks.AddRange(packResult.LoadedPacks);
                if (!packResult.IsSuccess)
                {
                    errors.AddRange(packResult.Errors);
                }
            }

            LastLoadErrors = errors.AsReadOnly();
            return errors.Count > 0
                ? ContentLoadResult.Partial(loadedPacks.AsReadOnly(), LastLoadErrors)
                : ContentLoadResult.Success(loadedPacks.AsReadOnly());
        }

        private List<(string Directory, PackManifest Manifest)> DiscoverPackManifests(
            string packsRootDirectory,
            List<string> errors)
        {
            List<(string Directory, PackManifest Manifest)> manifests = new List<(string Directory, PackManifest Manifest)>();

            foreach (string directory in _discoveryService.DiscoverPackDirectories(packsRootDirectory))
            {
                string manifestPath = Path.Combine(directory, "pack.yaml");
                try
                {
                    manifests.Add((directory, _packLoader.LoadFromFile(manifestPath)));
                }
                catch (Exception ex)
                {
                    errors.Add($"Failed to parse manifest in {directory}: {ex.Message}");
                }
            }

            return manifests;
        }

        private void LoadManifestContent(string packDirectory, PackManifest manifest, List<string> errors)
        {
            if (manifest.Loads != null)
            {
                LoadContentType(packDirectory, manifest, "units", manifest.Loads.Units, errors);
                LoadContentType(packDirectory, manifest, "buildings", manifest.Loads.Buildings, errors);
                LoadContentType(packDirectory, manifest, "factions", manifest.Loads.Factions, errors);
                LoadContentType(packDirectory, manifest, "weapons", manifest.Loads.Weapons, errors);
                LoadContentType(packDirectory, manifest, "doctrines", manifest.Loads.Doctrines, errors);
                LoadContentType(packDirectory, manifest, "faction_patches", manifest.Loads.FactionPatches, errors);

                List<string>? statsPaths = manifest.Overrides?.Stats?.Count > 0
                    ? manifest.Overrides.Stats
                    : null;
                LoadContentType(packDirectory, manifest, "stats", statsPaths, errors);
                return;
            }

            ScanConventionalDirectories(packDirectory, manifest, errors);
        }

        private void LoadContentType(
            string packDirectory,
            PackManifest manifest,
            string contentType,
            List<string>? declaredPaths,
            IList<string> errors)
        {
            IReadOnlyList<string> yamlFiles = _discoveryService.DiscoverYamlFiles(
                packDirectory,
                contentType,
                declaredPaths);

            foreach (string yamlFile in yamlFiles)
            {
                _registryImport.LoadAndRegisterContent(yamlFile, contentType, manifest, errors);
            }
        }

        private void ScanConventionalDirectories(string packDirectory, PackManifest manifest, IList<string> errors)
        {
            foreach (string contentType in _schemaResolver.ContentTypes)
            {
                LoadContentType(packDirectory, manifest, contentType, null, errors);
            }
        }

        /// <summary>
        /// Registers asset swaps in <see cref="DINOForge.SDK.Assets.AssetSwapRegistry"/> for all
        /// units and buildings loaded from this pack that declare a <c>visual_asset</c> field.
        /// The bundle path is resolved as <c>{packDirectory}/assets/bundles/{visual_asset}</c>.
        /// Registration is skipped silently when the bundle file does not exist on disk
        /// (e.g. bundles not yet built), so this method never blocks content loading.
        /// </summary>
        /// <param name="packDirectory">Absolute path to the pack root directory.</param>
        private void RegisterAssetSwaps(string packDirectory)
        {
            if (_registryManager == null)
                return;

            string bundlesDir = Path.Combine(packDirectory, "assets", "bundles");

            foreach (DINOForge.SDK.Registry.RegistryEntry<Models.UnitDefinition> entry in _registryManager.Units.All.Values)
            {
                Models.UnitDefinition unit = entry.Data;
                if (string.IsNullOrEmpty(unit.VisualAsset))
                    continue;

                string bundlePath = Path.Combine(bundlesDir, unit.VisualAsset!);
                if (!File.Exists(bundlePath))
                    continue;

                Assets.AssetSwapRegistry.Register(new Assets.AssetSwapRequest(
                    unit.VisualAsset!,
                    bundlePath,
                    unit.VisualAsset!,
                    unit.VanillaMapping));
            }

            foreach (DINOForge.SDK.Registry.RegistryEntry<Models.BuildingDefinition> entry in _registryManager.Buildings.All.Values)
            {
                Models.BuildingDefinition building = entry.Data;
                if (string.IsNullOrEmpty(building.VisualAsset))
                    continue;

                string bundlePath = Path.Combine(bundlesDir, building.VisualAsset!);
                if (!File.Exists(bundlePath))
                    continue;

                Assets.AssetSwapRegistry.Register(new Assets.AssetSwapRequest(
                    building.VisualAsset!,
                    bundlePath,
                    building.VisualAsset!));
            }
        }
    }
}
