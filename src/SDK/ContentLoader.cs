using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.SDK.Dependencies;
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
        /// <remarks>
        /// #837: Single-pack load mode cannot resolve <c>depends_on</c> references because no
        /// peer manifests are visible. If the manifest declares dependencies, this method fails
        /// fast with an error directing callers to <see cref="LoadPacks(string)"/>, which performs
        /// full dependency resolution via <see cref="PackDependencyResolver"/>.
        /// </remarks>
        /// <param name="packDirectory">Path to the pack directory.</param>
        /// <returns>Result indicating success or failure with errors.</returns>
        public ContentLoadResult LoadPack(string packDirectory)
        {
            return LoadPackInternal(packDirectory, skipDependencyCheck: false);
        }

        // #837 follow-up: LoadPacks() runs topological dependency resolution before invoking
        // per-pack load. When it calls into the single-pack path, the depends_on gate must NOT
        // fail-fast — peers will have been loaded by then via the ordered iteration. Surface
        // this distinction via an internal entry point rather than overloading public LoadPack.
        private ContentLoadResult LoadPackInternal(string packDirectory, bool skipDependencyCheck)
        {
            if (packDirectory == null)
            {
                throw new ArgumentNullException(nameof(packDirectory));
            }

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
                List<string> errors = new List<string> { $"Failed to parse manifest at {manifestPath}: {ex.Message}" };
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            // #762: Enforce framework_version compatibility. CheckPack uses
            // CompatibilityChecker.FrameworkVersion (resolved from SDK assembly) as the
            // installed SDK version. Errors abort the pack load; warnings are surfaced
            // through the load-errors collection but do not block.
            CompatibilityResult compatResult = CompatibilityChecker.CheckPack(manifest);
            if (!compatResult.IsCompatible)
            {
                List<string> errors = new List<string>
                {
                    $"Pack '{manifest.Id}' incompatible with DINOForge SDK {CompatibilityChecker.FrameworkVersion}: {string.Join("; ", compatResult.Errors)}"
                };
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            // #762 follow-up: Compatibility *warnings* are diagnostic — they do not affect
            // load success. Per ContentLoadResult contract, only Errors populate Errors[] and
            // gate IsSuccess. Warnings (e.g. unknown current BepInEx/Unity version when running
            // outside the game host) would otherwise cause every unit test pack to come back
            // as Partial (IsSuccess=False), regressing #523/#728/#762 tests.
            List<string> loadErrors = new List<string>();

            // #837: Single-pack load mode cannot resolve depends_on against an empty peer set.
            // Fail fast and direct callers to LoadPacks() which performs topological resolution.
            if (!skipDependencyCheck && manifest.DependsOn != null && manifest.DependsOn.Count > 0)
            {
                List<string> depErrors = new List<string>(loadErrors);
                foreach (string dep in manifest.DependsOn)
                {
                    depErrors.Add(
                        $"Pack '{manifest.Id}' declares dependency '{dep}' but was loaded via single-pack LoadPack(); " +
                        "dependencies cannot be resolved in single-pack mode. Use LoadPacks(packsRootDirectory) to load packs in dependency order.");
                }
                LastLoadErrors = depErrors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            LoadManifestContent(packDirectory, manifest, loadErrors);

            // Wire AssetSwapRegistry for all units and buildings with visual_asset references.
            RegisterAssetSwaps(packDirectory);

            LastLoadErrors = loadErrors.AsReadOnly();
            IReadOnlyList<string> loadedPackIds = new List<string> { manifest.Id }.AsReadOnly();

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

            manifests = DeduplicateManifestsByPackId(manifests, errors);
            DependencyResult dependencyResult = _dependencyResolver.ComputeLoadOrder(manifests.Select(item => item.Manifest));
            if (!dependencyResult.IsSuccess)
            {
                errors.AddRange(dependencyResult.Errors);
                LastLoadErrors = errors.AsReadOnly();
                return ContentLoadResult.Failure(LastLoadErrors);
            }

            errors.AddRange(_dependencyResolver.DetectConflicts(manifests.Select(item => item.Manifest)));

            // Build pack directory map, gracefully handling duplicate pack IDs by skipping duplicates with a warning.
            Dictionary<string, string> directoriesByPackId = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
            foreach (var item in manifests)
            {
                if (directoriesByPackId.ContainsKey(item.Manifest.Id))
                {
                    // Duplicate pack ID detected; skip the duplicate and log a warning.
                    errors.Add($"Duplicate pack ID '{item.Manifest.Id}' found in {item.Directory} — pack will be skipped.");
                }
                else
                {
                    directoriesByPackId[item.Manifest.Id] = item.Directory;
                }
            }

            List<string> loadedPacks = new List<string>();
            foreach (PackManifest orderedManifest in dependencyResult.LoadOrder)
            {
                if (!directoriesByPackId.TryGetValue(orderedManifest.Id, out string? packDirectory))
                {
                    continue;
                }

                ContentLoadResult packResult = LoadPackInternal(packDirectory, skipDependencyCheck: true);
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

        /// <summary>
        /// Collapses duplicate pack IDs (e.g. <c>economy-balanced</c> and <c>economy-balanced.disabled</c>)
        /// before dependency resolution. Prefers directories whose folder name does not end with
        /// <c>.disabled</c>.
        /// </summary>
        private static List<(string Directory, PackManifest Manifest)> DeduplicateManifestsByPackId(
            List<(string Directory, PackManifest Manifest)> manifests,
            List<string> errors)
        {
            Dictionary<string, (string Directory, PackManifest Manifest)> byId =
                new Dictionary<string, (string Directory, PackManifest Manifest)>(StringComparer.OrdinalIgnoreCase);

            foreach ((string directory, PackManifest manifest) in manifests)
            {
                if (!byId.TryGetValue(manifest.Id, out (string Directory, PackManifest Manifest) existing))
                {
                    byId[manifest.Id] = (directory, manifest);
                    continue;
                }

                bool existingIsDisabled = Path.GetFileName(existing.Directory)
                    .EndsWith(".disabled", StringComparison.OrdinalIgnoreCase);
                bool candidateIsDisabled = Path.GetFileName(directory)
                    .EndsWith(".disabled", StringComparison.OrdinalIgnoreCase);

                if (existingIsDisabled && !candidateIsDisabled)
                {
                    errors.Add(
                        $"Duplicate pack ID '{manifest.Id}' in {existing.Directory} — using {directory} instead.");
                    byId[manifest.Id] = (directory, manifest);
                }
                else if (!existingIsDisabled && candidateIsDisabled)
                {
                    errors.Add(
                        $"Duplicate pack ID '{manifest.Id}' in {directory} — pack will be skipped.");
                }
                else
                {
                    errors.Add(
                        $"Duplicate pack ID '{manifest.Id}' in {directory} — pack will be skipped.");
                }
            }

            return byId.Values.ToList();
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
