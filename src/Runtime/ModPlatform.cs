#nullable enable
// Iter-144 #543 gray-freeze patch — pre-existing DF analyzer warnings in this file are
// outside the scope of the patch and tracked separately (see Pattern Catalog #106/#231).
#pragma warning disable DF0106 // implicit File.ReadAllText encoding (pre-existing, tracked)
#pragma warning disable DF1006 // disposable field (pre-existing BepInEx-owned, tracked)
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using Newtonsoft.Json;
using BepInEx;
using BepInEx.Configuration;
using BepInEx.Logging;
using DINOForge.Runtime.Bridge;
using DINOForge.Runtime.HotReload;
using DINOForge.Runtime.UI;
using DINOForge.SDK;
using DINOForge.SDK.HotReload;
using DINOForge.SDK.Registry;
using Unity.Entities;
using UnityEngine;

namespace DINOForge.Runtime
{
    /// <summary>
    /// Central orchestrator for the DINOForge mod platform. Coordinates pack loading,
    /// registry population, ECS system registration, UI overlays, and hot reload.
    /// This is NOT a MonoBehaviour; it is owned by <see cref="Plugin"/>.
    /// </summary>
    public sealed class ModPlatform
    {
        private ManualLogSource _log = null!;
        private ConfigFile _config = null!;
        private GameObject _pluginObject = null!;

        // Config entries
        private ConfigEntry<string> _packsDirectory = null!;
        private ConfigEntry<bool> _autoLoadOnStartup = null!;
        private ConfigEntry<bool> _hotReloadEnabled = null!;

        // Subsystems
        private RegistryManager? _registryManager;
        private ContentLoader? _contentLoader;
        private VanillaCatalog? _vanillaCatalog;

        // UI
        private IModMenuHost? _modMenuHost;
        private IModSettingsHost? _modSettingsHost;

        // Hot reload
        private PackFileWatcher? _packFileWatcher;
        private HotReloadBridge? _hotReloadBridge;

        // IPC
        private GameBridgeServer? _gameBridgeServer;

        // State
        private bool _initialized;
        private bool _worldReady;
        private ContentLoadResult? _lastLoadResult;
        private readonly Dictionary<string, CachedPackDisplayInfo> _packDisplayInfoCache =
            new Dictionary<string, CachedPackDisplayInfo>(StringComparer.OrdinalIgnoreCase);
        // Pattern #99: pack IDs are schema-driven (YAML manifest), ordinal-comparison-only.
        // Keep ALL pack-ID lookups (HashSet, Equals, Dictionary) on StringComparer.Ordinal
        // to avoid drift between case-sensitive and case-insensitive paths (see L703 fix).
        private readonly HashSet<string> _disabledPacks = new HashSet<string>(StringComparer.Ordinal);
        private const string DisabledPacksFile = "disabled_packs.json";

        private sealed class CachedPackDisplayInfo
        {
            public CachedPackDisplayInfo(DateTime lastWriteUtc, long length, PackDisplayInfo displayInfo)
            {
                LastWriteUtc = lastWriteUtc;
                Length = length;
                DisplayInfo = displayInfo;
            }

            public DateTime LastWriteUtc { get; }

            public long Length { get; }

            public PackDisplayInfo DisplayInfo { get; }
        }

        /// <summary>The registry manager containing all loaded content.</summary>
        public RegistryManager? Registry => _registryManager;

        /// <summary>
        /// Invoked after every pack load (initial and reload) with (packCount, errorCount).
        /// Wire this to <see cref="UI.HudStrip.SetStatus"/> or <see cref="UI.HudIndicator.UpdateCounts"/>
        /// from the active UI layer so the HUD counter stays in sync.
        /// </summary>
        public Action<int, int>? OnHudCountsChanged;

        /// <summary>The vanilla entity catalog built from the ECS world.</summary>
        public VanillaCatalog? Catalog => _vanillaCatalog;

        /// <summary>The content loader for pack loading operations.</summary>
        public ContentLoader? ContentLoader => _contentLoader;

        /// <summary>The configured packs directory path.</summary>
        public string PacksDirectory => _packsDirectory?.Value ?? "";

        /// <summary>Whether the platform has been initialized.</summary>
        public bool IsInitialized => _initialized;

        /// <summary>Whether the ECS world is ready and systems are registered.</summary>
        public bool IsWorldReady => _worldReady;

        /// <summary>Returns the IDs of all currently loaded packs (thread-safe read).</summary>
        public IReadOnlyList<string>? GetLoadedPackIds() => _lastLoadResult?.LoadedPacks;

        /// <summary>
        /// Builds the current pack list for UI presentation from the latest load result.
        /// Returns an empty list if packs have not been loaded yet or the registry is unavailable.
        /// </summary>
        public IReadOnlyList<PackDisplayInfo> GetLoadedPackDisplayInfos()
        {
            if (_lastLoadResult == null || _registryManager == null)
            {
                return Array.Empty<PackDisplayInfo>();
            }

            return BuildPackDisplayInfos(_lastLoadResult);
        }

        /// <summary>Returns the last pack load result (including errors) for UI display.</summary>
        internal ContentLoadResult? GetLastLoadResult() => _lastLoadResult;

        /// <summary>Returns whether the last pack load result is available for diagnostics.</summary>
        internal bool HasLastLoadResult => _lastLoadResult != null;

        /// <summary>Describes the last pack load result for diagnostics.</summary>
        internal string DescribeLastLoadResult()
        {
            if (_lastLoadResult == null)
            {
                return "lastLoadResult=NULL";
            }

            return $"lastLoadResult=present success={_lastLoadResult.IsSuccess} loaded={_lastLoadResult.LoadedPacks.Count} errors={_lastLoadResult.Errors.Count}";
        }

        /// <summary>
        /// Initializes the mod platform with all subsystems.
        /// Call this from <see cref="Plugin.Awake"/>.
        /// </summary>
        /// <param name="log">BepInEx logger.</param>
        /// <param name="config">BepInEx config file for storing settings.</param>
        /// <param name="pluginObject">The plugin's GameObject (for adding MonoBehaviour components).</param>
        public void Initialize(ManualLogSource log, ConfigFile config, GameObject pluginObject)
        {
            if (_initialized)
            {
                log.LogWarning("[ModPlatform] Already initialized, skipping.");
                return;
            }

            _log = log ?? throw new ArgumentNullException(nameof(log));
            _config = config ?? throw new ArgumentNullException(nameof(config));
            _pluginObject = pluginObject ?? throw new ArgumentNullException(nameof(pluginObject));

            _log.LogInfo("[ModPlatform] Initializing...");

            // Bind config entries
            try
            {
                _packsDirectory = _config.Bind(
                    "Packs", "PacksDirectory",
                    Path.Combine(Paths.BepInExRootPath, "dinoforge_packs"),
                    "Directory containing DINOForge content packs");

                _autoLoadOnStartup = _config.Bind(
                    "Packs", "AutoLoadOnStartup",
                    true,
                    "Automatically load all packs when the game starts");

                _hotReloadEnabled = _config.Bind(
                    "HotReload", "Enabled",
                    true,
                    "Watch pack files for changes and reload automatically");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Config binding failed: {ex}");
                return;
            }

            // Create core subsystems
            try
            {
                _registryManager = new RegistryManager();
                _contentLoader = new ContentLoader(
                    _registryManager,
                    schemaValidator: null,
                    log: msg => _log.LogInfo(msg));

                _vanillaCatalog = new VanillaCatalog();

                _log.LogInfo("[ModPlatform] Core subsystems created.");

                // Load disabled packs from disk
                LoadDisabledPacks();
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to create subsystems: {ex}");
                return;
            }

            // Ensure packs directory exists
            try
            {
                string packsDir = _packsDirectory.Value;
                if (!Directory.Exists(packsDir))
                {
                    Directory.CreateDirectory(packsDir);
                    _log.LogInfo($"[ModPlatform] Created packs directory: {packsDir}");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] Could not create packs directory: {ex}");
            }

            _initialized = true;
            _log.LogInfo("[ModPlatform] Initialization complete.");
        }

        /// <summary>
        /// Called when the ECS World becomes available. Registers ECS systems
        /// and builds the vanilla entity catalog.
        /// </summary>
        /// <param name="world">The default ECS world.</param>
        public void OnWorldReady(World world)
        {
            if (!_initialized)
            {
                _log.LogError("[ModPlatform] Cannot process world - not initialized.");
                return;
            }

            if (_worldReady)
            {
                _log.LogWarning("[ModPlatform] World already processed, skipping.");
                return;
            }

            _log.LogInfo($"[ModPlatform] ECS World ready: {world.Name}");

            // Register the StatModifierSystem
            try
            {
                world.GetOrCreateSystem<StatModifierSystem>();
                _log.LogInfo("[ModPlatform] StatModifierSystem registered.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to register StatModifierSystem: {ex}");
            }

            // Register the PackUnitSpawner
            try
            {
                world.GetOrCreateSystem<PackUnitSpawner>();
                _log.LogInfo("[ModPlatform] PackUnitSpawner registered.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to register PackUnitSpawner: {ex}");
            }

            // Register the WaveInjector
            try
            {
                world.GetOrCreateSystem<WaveInjector>();
                WaveInjector.SetRegistryManager(_registryManager!);
                _log.LogInfo("[ModPlatform] WaveInjector registered.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] WaveInjector failed: {ex}");
            }

            // Register the FactionSystem
            try
            {
                world.GetOrCreateSystem<FactionSystem>();
                if (_registryManager != null)
                    FactionSystem.InitializeFactions(_registryManager.Factions);
                _log.LogInfo("[ModPlatform] FactionSystem initialized.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] FactionSystem failed: {ex}");
            }

            // Build the vanilla entity catalog
            // Iter-144 #543: short-circuit if world is being torn down (gray-freeze fix).
            if (RuntimeDriver.IsBeingDestroyed)
            {
                _log.LogWarning("[ModPlatform] Skipping VanillaCatalog.Build — RuntimeDriver.IsBeingDestroyed=true (scene teardown in progress).");
            }
            else
            {
                try
                {
                    // null-forgiveness-ok: _vanillaCatalog set in Initialize before any Build call
                    _vanillaCatalog!.Build(world.EntityManager);
                    _log.LogInfo($"[ModPlatform] VanillaCatalog built: " +
                        $"{_vanillaCatalog.Units.Count} units, " +
                        $"{_vanillaCatalog.Buildings.Count} buildings, " +
                        $"{_vanillaCatalog.Projectiles.Count} projectiles.");
                }
                catch (Exception ex)
                {
                    // Iter-144 #543: Pattern #96 — surface full exception detail. Previous silent
                    // swallow masked the ArgumentNullException(MemSet destination=null) race that
                    // caused the gray-freeze hang. Future world-teardown races will be visible.
                    _log.LogError($"[ModPlatform] VanillaCatalog build failed: {ex}");
                    if (ex.InnerException != null)
                    {
                        _log.LogError($"[ModPlatform] VanillaCatalog inner: {ex.InnerException}");
                    }
                }
            }

            // Validate component mappings
            try
            {
                (int resolved, int total, List<string> unresolved) = ComponentMap.ValidateResolution();
                _log.LogInfo($"[ModPlatform] ComponentMap: {resolved}/{total} types resolved.");
                foreach (string unresolvedType in unresolved)
                {
                    _log.LogWarning($"[ModPlatform] Unresolved component type: {unresolvedType}");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] ComponentMap validation failed: {ex}");
            }

            // Start/reuse the IPC bridge server (static singleton on Plugin to survive scene transitions)
            try
            {
                if (Plugin.SharedBridgeServer == null)
                {
                    var bridge = new GameBridgeServer(this);
                    bridge.Start();
                    Plugin.SharedBridgeServer = bridge;
                    _log.LogInfo("[ModPlatform] GameBridgeServer started (new singleton).");
                }
                else
                {
                    Plugin.SharedBridgeServer.UpdatePlatform(this);
                    _log.LogInfo("[ModPlatform] GameBridgeServer reattached to new ModPlatform.");
                }
                _gameBridgeServer = Plugin.SharedBridgeServer;
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to start GameBridgeServer: {ex}");
            }

            _worldReady = true;
        }

        /// <summary>
        /// Rebuilds VanillaCatalog against the live ECS world (must have >1000 entities)
        /// and re-triggers stat modifier application. Called once the game scene is loaded.
        /// </summary>
        public void RebuildCatalogAndApplyStats(Unity.Entities.World world)
        {
            // Iter-144 #543 gray-freeze fix: short-circuit if RuntimeDriver is being destroyed,
            // so pack stat re-injection does not race world teardown.
            if (RuntimeDriver.IsBeingDestroyed)
            {
                _log.LogWarning("[ModPlatform] Skipping RebuildCatalogAndApplyStats — RuntimeDriver.IsBeingDestroyed=true.");
                return;
            }

            try
            {
                _vanillaCatalog!.Build(world.EntityManager);
                _log.LogInfo($"[ModPlatform] VanillaCatalog rebuilt: " +
                    $"{_vanillaCatalog.Units.Count} units, " +
                    $"{_vanillaCatalog.Buildings.Count} buildings, " +
                    $"{_vanillaCatalog.Projectiles.Count} projectiles.");
            }
            catch (Exception ex)
            {
                // Iter-144 #543: Pattern #96 — surface full exception detail.
                _log.LogError($"[ModPlatform] VanillaCatalog rebuild failed: {ex}");
                return;
            }

            // Apply pack unit stat definitions to matching vanilla ECS entities.
            // PackStatInjector replaces the no-op ApplyUnitOverrides path for vanilla_mapping units.
            if (_registryManager != null)
            {
                try
                {
                    int injectedWrites = PackStatInjector.Apply(
                        world.EntityManager,
                        _registryManager,
                        msg => _log.LogInfo(msg));
                    _log.LogInfo($"[ModPlatform] PackStatInjector: {injectedWrites} entity-field write(s) applied.");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[ModPlatform] PackStatInjector failed: {ex}");
                }
            }

            // Re-enqueue global YAML stat overrides now that the catalog is populated
            if (_registryManager != null && _contentLoader != null)
            {
                try
                {
                    int unitOverrides = OverrideApplicator.ApplyUnitOverrides(_registryManager, msg => _log.LogInfo(msg));
                    _log.LogInfo($"[ModPlatform] Re-enqueued {unitOverrides} unit stat override(s) after scene load.");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[ModPlatform] Unit stat override re-apply failed: {ex}");
                }

                try
                {
                    if (_contentLoader.LoadedOverrides.Count > 0)
                    {
                        int yamlOverrides = OverrideApplicator.ApplyStatOverrides(_contentLoader.LoadedOverrides, msg => _log.LogInfo(msg));
                        _log.LogInfo($"[ModPlatform] Re-enqueued {yamlOverrides} YAML stat override(s) after scene load.");
                    }
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[ModPlatform] YAML stat override re-apply failed: {ex}");
                }
            }
        }

        /// <summary>
        /// Loads all content packs from the configured packs directory.
        /// After loading, updates the UI overlay and enqueues stat modifications.
        /// </summary>
        /// <returns>The result of the load operation.</returns>
        public ContentLoadResult LoadPacks()
        {
            // Iter-144 H9 probe: ENTER/EXIT timing around mod-side pack-load entry.
            var __h9sw = System.Diagnostics.Stopwatch.StartNew();
            _log?.LogInfo($"[ModPlatform.LoadPacks] ENTER thread={System.Threading.Thread.CurrentThread.ManagedThreadId}");
            try
            {
                return LoadPacksImpl();
            }
            finally
            {
                _log?.LogInfo($"[ModPlatform.LoadPacks] EXIT elapsed={__h9sw.ElapsedMilliseconds}ms");
            }
        }

        private ContentLoadResult LoadPacksImpl()
        {
            // Iter-144 #547 H6 gray-freeze fix: short-circuit if RuntimeDriver is being destroyed.
            // Pack-load can race scene teardown — a new RuntimeDriver may attempt LoadPacks while
            // the previous one's OnDestroy chain is still running and disposing shared state
            // (PackFileWatcher, HotReloadBridge, AssetBundleCache). Cleanly skipping prevents
            // partial state corruption that wedged the main thread post-OnDestroy.
            if (RuntimeDriver.IsBeingDestroyed)
            {
                _log?.LogWarning("[ModPlatform] Skipping LoadPacks — RuntimeDriver.IsBeingDestroyed=true (H6 race guard).");
                return ContentLoadResult.Failure(
                    new List<string> { "LoadPacks aborted — RuntimeDriver being destroyed" }.AsReadOnly());
            }
            if (!_initialized || _contentLoader == null || _registryManager == null)
            {
                _log.LogError("[ModPlatform] Cannot load packs - not initialized.");
                return ContentLoadResult.Failure(
                    new List<string> { "ModPlatform not initialized" }.AsReadOnly());
            }

            string packsDir = _packsDirectory.Value;
            _log.LogInfo($"[ModPlatform] Loading packs from: {packsDir}");

            // Temporarily disable packs by renaming directories
            List<string> temporarilyDisabledDirs = new List<string>();
            if (_disabledPacks.Count > 0)
            {
                foreach (string packId in _disabledPacks)
                {
                    string packPath = Path.Combine(packsDir, packId);
                    if (Directory.Exists(packPath))
                    {
                        string disabledPath = packPath + ".disabled";
                        try
                        {
                            Directory.Move(packPath, disabledPath);
                            temporarilyDisabledDirs.Add(packPath);
                            _log.LogInfo($"[ModPlatform] Temporarily disabled pack: {packId}");
                        }
                        catch (Exception ex)
                        {
                            _log.LogWarning($"[ModPlatform] Failed to disable pack {packId}: {ex}");
                        }
                    }
                }
            }

            ContentLoadResult result;
            try
            {
                result = _contentLoader.LoadPacks(packsDir);
                _lastLoadResult = result;
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Pack loading failed: {ex}");
                result = ContentLoadResult.Failure(
                    new List<string> { $"Pack loading exception: {ex.Message}" }.AsReadOnly());
                _lastLoadResult = result;
                UpdateUI(result);
                return result;
            }
            finally
            {
                // Re-enable temporarily disabled packs
                foreach (string originalPath in temporarilyDisabledDirs)
                {
                    string disabledPath = originalPath + ".disabled";
                    try
                    {
                        if (Directory.Exists(disabledPath))
                        {
                            Directory.Move(disabledPath, originalPath);
                            _log.LogInfo($"[ModPlatform] Re-enabled pack: {Path.GetFileName(originalPath)}");
                        }
                    }
                    catch (Exception ex)
                    {
                        _log.LogWarning($"[ModPlatform] Failed to re-enable pack {originalPath}: {ex}");
                    }
                }
            }

            // Log results
            if (result.IsSuccess)
            {
                _log.LogInfo($"[ModPlatform] Successfully loaded {result.LoadedPacks.Count} pack(s).");
            }
            else
            {
                _log.LogWarning($"[ModPlatform] Loaded {result.LoadedPacks.Count} pack(s) with {result.Errors.Count} error(s).");
                foreach (string error in result.Errors)
                {
                    _log.LogError($"  {error}");
                }
            }

            // Initialize PackUnitSpawner with the registry
            try
            {
                PackUnitSpawner.Initialize(_registryManager);
                _log.LogInfo("[ModPlatform] PackUnitSpawner initialized with registry.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to initialize PackUnitSpawner: {ex}");
            }

            // Initialize AerialSpawnSystem so it can sweep baked building entities for
            // defense_tags: [AntiAir] and attach AntiAirComponent on its startup pass.
            try
            {
                DINOForge.Runtime.Aviation.AerialSpawnSystem.Initialize(_registryManager);
                _log.LogInfo("[ModPlatform] AerialSpawnSystem initialized with building registry.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to initialize AerialSpawnSystem: {ex}");
            }

            // Apply stat overrides from loaded units
            try
            {
                int overrideCount = OverrideApplicator.ApplyUnitOverrides(
                    _registryManager,
                    msg => _log.LogInfo(msg));
                _log.LogInfo($"[ModPlatform] {overrideCount} stat override(s) enqueued.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Stat override application failed: {ex}");
            }

            // Apply YAML stat overrides
            try
            {
                if (_contentLoader.LoadedOverrides.Count > 0)
                {
                    int statOverrideCount = OverrideApplicator.ApplyStatOverrides(
                        _contentLoader.LoadedOverrides,
                        msg => _log.LogInfo(msg));
                    _log.LogInfo($"[ModPlatform] {statOverrideCount} YAML stat override(s) enqueued.");
                }
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] YAML stat override application failed: {ex}");
            }

            // Update UI
            UpdateUI(result);

            return result;
        }

        /// <summary>
        /// Creates and starts the hot reload system (PackFileWatcher + HotReloadBridge).
        /// </summary>
        public void StartHotReload()
        {
            if (!_initialized || _contentLoader == null || _registryManager == null)
            {
                _log.LogError("[ModPlatform] Cannot start hot reload - not initialized.");
                return;
            }

            if (!_hotReloadEnabled.Value)
            {
                _log.LogInfo("[ModPlatform] Hot reload disabled in config.");
                return;
            }

            string packsDir = _packsDirectory.Value;

            try
            {
                // #611: PackFileWatcher debounce defaults to 15000ms (15s) per SDK convention.
                // Previously this passed 500ms explicitly which thrashed pack-reload on rapid
                // editor saves. Production hot-reload uses the SDK default; tests pass
                // shorter values explicitly (HotReloadTests: 50/100/200ms).
                _packFileWatcher = new PackFileWatcher(
                    packsDir,
                    _contentLoader,
                    _registryManager,
                    schemaValidator: null,
                    log: msg => _log.LogInfo(msg));

                _hotReloadBridge = new HotReloadBridge(
                    _packFileWatcher,
                    _registryManager,
                    _log);

#pragma warning disable DF0105
                // Wire up events: when hot reload updates, re-apply overrides and refresh UI
                _hotReloadBridge.OnRuntimeUpdated += OnHotReloadCompleted;
#pragma warning restore DF0105

                _hotReloadBridge.Start();
                _log.LogInfo($"[ModPlatform] Hot reload started, watching: {packsDir}");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to start hot reload: {ex}");
            }
        }

        /// <summary>
        /// Handles hot reload completion by re-applying stat overrides and updating UI.
        /// </summary>
        private void OnHotReloadCompleted(object? sender, HotReloadResult result)
        {
            try
            {
                _log.LogInfo($"[ModPlatform] Hot reload completed. " +
                    $"Changed: {result.ChangedFiles.Count}, Updated: {result.UpdatedEntries.Count}");

                // Re-apply stat overrides
                if (_registryManager != null)
                {
                    int overrideCount = OverrideApplicator.ApplyUnitOverrides(
                        _registryManager,
                        msg => _log.LogInfo(msg));
                    _log.LogInfo($"[ModPlatform] Re-applied {overrideCount} stat override(s) after hot reload.");

                    if (_contentLoader != null && _contentLoader.LoadedOverrides.Count > 0)
                    {
                        OverrideApplicator.ApplyStatOverrides(_contentLoader.LoadedOverrides, msg => _log.LogInfo(msg));
                    }

                    // Tell StatModifierSystem to re-process
                    StatModifierSystem.Reapply();

                    // If any changed files are bundle assets, schedule a full swap reset so
                    // the new bundle bytes are picked up on the next game/save load (without
                    // requiring a full game restart).
                    bool bundleChanged = false;
                    foreach (string changedFile in result.ChangedFiles)
                    {
                        if (changedFile.IndexOf("assets/bundles", StringComparison.OrdinalIgnoreCase) >= 0
                            || changedFile.IndexOf(@"assets\bundles", StringComparison.OrdinalIgnoreCase) >= 0)
                        {
                            bundleChanged = true;
                            break;
                        }
                    }
                    if (bundleChanged)
                    {
                        AssetSwapSystem.ScheduleReset();
                        _log.LogInfo("[ModPlatform] Bundle change detected — asset swap reset scheduled for next load.");
                    }
                }

                // Update UI with current state
                if (_lastLoadResult != null)
                {
                    UpdateUI(_lastLoadResult);
                }
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Error handling hot reload completion: {ex}");
            }
        }

        /// <summary>
        /// Updates the active mod-menu host with current pack information and status.
        /// </summary>
        private void UpdateUI(ContentLoadResult result)
        {
            if (_modMenuHost == null || _registryManager == null) return;

            try
            {
                _modMenuHost.SetPacks(BuildPackDisplayInfos(result));

                // Set status message — include first error detail so it's visible without logs.
                string statusMsg;
                if (result.IsSuccess)
                {
                    statusMsg = $"All {result.LoadedPacks.Count} pack(s) loaded OK";
                }
                else
                {
                    string detail = result.Errors.Count > 0 ? $": {result.Errors[0]}" : string.Empty;
                    statusMsg = $"{result.LoadedPacks.Count} loaded, {result.Errors.Count} error(s){detail}";
                }
                _modMenuHost.SetStatus(statusMsg, result.Errors.Count);

                // Sync HUD strip / IMGUI indicator pack count.
                OnHudCountsChanged?.Invoke(result.LoadedPacks.Count, result.Errors.Count);
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] UI update failed: {ex}");
            }
        }

        private IReadOnlyList<PackDisplayInfo> BuildPackDisplayInfos(ContentLoadResult result)
        {
            // Build PackDisplayInfo list from the registry manager's loaded content.
            // We need to re-read manifests since ContentLoadResult only has IDs.
            List<PackDisplayInfo> packInfos = new List<PackDisplayInfo>();

            // Use the packs directory to find manifests for display.
            string packsDir = _packsDirectory.Value;
            if (Directory.Exists(packsDir))
            {
                PackLoader packLoader = new PackLoader();
                foreach (string dir in Directory.GetDirectories(packsDir))
                {
                    string manifestPath = Path.Combine(dir, "pack.yaml");
                    if (!File.Exists(manifestPath)) continue;

                    try
                    {
                        PackDisplayInfo displayInfo = GetCachedPackDisplayInfo(manifestPath, packLoader);
                        bool isLoaded = result.LoadedPacks.Any(
                            loadedId => string.Equals(loadedId, displayInfo.Id, StringComparison.Ordinal));
                        bool isDisabled = _disabledPacks.Contains(displayInfo.Id);

                        packInfos.Add(displayInfo.WithEnabled(isLoaded && !isDisabled));
                    }
                    catch (Exception ex)
                    {
                        _log.LogWarning($"[ModPlatform] Could not read manifest in {dir}: {ex}");
                    }
                }
            }

            HashSet<string> displayedIds = new HashSet<string>(
                packInfos.Select(pack => pack.Id),
                StringComparer.Ordinal);

            foreach (string loadedId in result.LoadedPacks)
            {
                if (displayedIds.Contains(loadedId))
                {
                    continue;
                }

                bool isDisabled = _disabledPacks.Contains(loadedId);
                packInfos.Add(new PackDisplayInfo(
                    id: loadedId,
                    name: loadedId,
                    version: "unknown",
                    author: "unknown",
                    type: "pack",
                    description: "Loaded pack metadata was not found on disk.",
                    loadOrder: 0,
                    isEnabled: !isDisabled,
                    dependencies: Array.Empty<string>(),
                    conflicts: Array.Empty<string>(),
                    errors: new List<string>().AsReadOnly()));
            }

            DetectContentConflicts(packInfos);

            return packInfos;
        }

        private static Dictionary<string, int> ExtractContentSummary(PackManifest manifest)
        {
            var summary = new Dictionary<string, int>(StringComparer.Ordinal);
            if (manifest.Loads == null) return summary;

            void Add(string key, List<string>? items)
            {
                if (items != null && items.Count > 0)
                    summary[key] = items.Count;
            }

            Add("factions", manifest.Loads.Factions);
            Add("units", manifest.Loads.Units);
            Add("buildings", manifest.Loads.Buildings);
            Add("weapons", manifest.Loads.Weapons);
            Add("doctrines", manifest.Loads.Doctrines);
            Add("scenarios", manifest.Loads.Scenarios);
            Add("wave_templates", manifest.Loads.WaveTemplates);
            Add("tech_nodes", manifest.Loads.TechNodes);
            Add("audio", manifest.Loads.Audio);
            Add("visuals", manifest.Loads.Visuals);
            Add("localization", manifest.Loads.Localization);
            Add("faction_patches", manifest.Loads.FactionPatches);
            // Fields below don't exist on PackLoads yet — add when SDK PackManifest gains them

            if (manifest.Overrides != null)
            {
                void AddOverride(string key, List<string>? items)
                {
                    if (items != null && items.Count > 0)
                    {
                        string overrideKey = key + " (overrides)";
                        summary[overrideKey] = items.Count;
                    }
                }
                AddOverride("units", manifest.Overrides.Units);
                AddOverride("buildings", manifest.Overrides.Buildings);
                AddOverride("stats", manifest.Overrides.Stats);
            }

            return summary;
        }

        private static void DetectContentConflicts(List<PackDisplayInfo> packs)
        {
            var contentTypeOwners = new Dictionary<string, List<string>>(StringComparer.Ordinal);

            foreach (PackDisplayInfo pack in packs)
            {
                if (!pack.IsEnabled) continue;
                foreach (string contentType in pack.ContentSummary.Keys)
                {
                    if (!contentTypeOwners.TryGetValue(contentType, out List<string>? owners))
                    {
                        owners = new List<string>();
                        contentTypeOwners[contentType] = owners;
                    }
                    owners.Add(pack.Id);
                }
            }

            for (int i = 0; i < packs.Count; i++)
            {
                PackDisplayInfo pack = packs[i];
                if (pack.ContentSummary.Count == 0) continue;

                var conflicts = new List<string>();
                foreach (string contentType in pack.ContentSummary.Keys)
                {
                    if (!contentTypeOwners.TryGetValue(contentType, out List<string>? owners)) continue;
                    foreach (string otherId in owners)
                    {
                        if (string.Equals(otherId, pack.Id, StringComparison.Ordinal)) continue;
                        conflicts.Add($"{otherId} also loads: {contentType}");
                    }
                }

                if (conflicts.Count > 0)
                {
                    packs[i] = new PackDisplayInfo(
                        pack.Id, pack.Name, pack.Version, pack.Author, pack.Type,
                        pack.Description, pack.LoadOrder, pack.IsEnabled,
                        pack.Dependencies, pack.Conflicts, pack.Errors,
                        pack.ContentSummary, conflicts.AsReadOnly());
                }
            }
        }

        private PackDisplayInfo GetCachedPackDisplayInfo(string manifestPath, PackLoader packLoader)
        {
            FileInfo manifestFile = new FileInfo(manifestPath);
            if (_packDisplayInfoCache.TryGetValue(manifestPath, out CachedPackDisplayInfo? cached)
                && cached.LastWriteUtc == manifestFile.LastWriteTimeUtc
                && cached.Length == manifestFile.Length)
            {
                return cached.DisplayInfo;
            }

            PackManifest manifest = packLoader.LoadFromFile(manifestPath);
            Dictionary<string, int> contentSummary = ExtractContentSummary(manifest);
            PackDisplayInfo displayInfo = new PackDisplayInfo(
                id: manifest.Id,
                name: manifest.Name,
                version: manifest.Version,
                author: manifest.Author,
                type: manifest.Type,
                description: manifest.Description,
                loadOrder: manifest.LoadOrder,
                isEnabled: true,
                dependencies: manifest.DependsOn.AsReadOnly(),
                conflicts: manifest.ConflictsWith.AsReadOnly(),
                errors: new List<string>().AsReadOnly(),
                contentSummary: contentSummary);

            _packDisplayInfoCache[manifestPath] = new CachedPackDisplayInfo(
                manifestFile.LastWriteTimeUtc,
                manifestFile.Length,
                displayInfo);

            return displayInfo;
        }

        /// <summary>
        /// Sets the UI overlay references. Called by Plugin after adding components to the GameObject.
        /// </summary>
        /// <param name="menuHost">The active mod menu host.</param>
        /// <param name="settingsHost">The active mod settings host.</param>
        public void SetUI(IModMenuHost menuHost, IModSettingsHost settingsHost)
        {
            _modMenuHost = menuHost;
            _modSettingsHost = settingsHost;

            // Wire reload button to hot reload
            if (_modMenuHost != null)
            {
                _modMenuHost.OnReloadRequested = OnReloadRequested;
                _modMenuHost.OnPackToggled = OnPackToggled;
            }
        }

        /// <summary>
        /// #874: Public accessor for unifying HMR signal-file pipeline with PackFileWatcher pipeline.
        /// Returns true if HotReloadBridge ran (which fires StatModifierSystem.Reapply + OnRuntimeUpdated).
        /// Falls back to LoadPacks() if bridge unavailable.
        /// </summary>
        public bool TriggerHotReload()
        {
            try
            {
                if (_hotReloadBridge != null)
                {
                    HotReloadResult result = _hotReloadBridge.TriggerReload();
                    _log.LogInfo($"[ModPlatform] TriggerHotReload (HMR signal): success={result.IsSuccess}");
                    if (result.IsSuccess)
                    {
                        LoadPacks();
                    }
                    return true;
                }
                LoadPacks();
                return false;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] TriggerHotReload failed: {ex}");
                return false;
            }
        }

        /// <summary>
        /// Handles the reload button press from the mod menu overlay.
        /// </summary>
        private void OnReloadRequested()
        {
            _log.LogInfo("[ModPlatform] Reload requested from UI.");

            try
            {
                if (_hotReloadBridge != null)
                {
                    // Use hot reload bridge for registry updates
                    HotReloadResult result = _hotReloadBridge.TriggerReload();
                    _log.LogInfo($"[ModPlatform] UI-triggered reload: success={result.IsSuccess}");

                    // Refresh UI pack list to show latest state from disk after hot reload
                    if (result.IsSuccess)
                    {
                        LoadPacks();
                    }
                }
                else
                {
                    // Fallback: just reload packs directly
                    LoadPacks();
                }
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Reload failed: {ex}");
                _modMenuHost?.SetStatus($"Reload failed: {ex.Message}", 1);
            }
        }

        /// <summary>
        /// Handles pack toggle events from the UI overlay.
        /// Changes the enabled state and immediately reloads packs to apply the toggle.
        /// </summary>
        private void OnPackToggled(string packId, bool enabled)
        {
            _log.LogInfo($"[ModPlatform] Pack '{packId}' toggled: enabled={enabled}");
            SetPackEnabled(packId, enabled);

            // Immediately apply the toggle by reloading packs for legacy hosts. RuntimeDriver
            // overrides UGUI callbacks with a deferred queue so Unity button presses never run
            // pack IO on the click stack.
            try
            {
                _log.LogInfo($"[ModPlatform] Reloading packs after toggle...");
                LoadPacks();
                _modMenuHost?.SetStatus($"Pack '{packId}' {(enabled ? "enabled" : "disabled")} and reloaded");
            }
            catch (Exception ex)
            {
                _log.LogError($"[ModPlatform] Failed to reload after toggle: {ex}");
                _modMenuHost?.SetStatus($"Reload after toggle failed: {ex.Message}", 1);
            }
        }

        /// <summary>
        /// Updates the persisted enabled state for a pack without reloading content.
        /// The runtime driver uses this from its deferred UI action queue.
        /// </summary>
        public void SetPackEnabled(string packId, bool enabled)
        {
            if (enabled)
            {
                _disabledPacks.Remove(packId);
                _log.LogInfo($"[ModPlatform] Pack '{packId}' enabled");
            }
            else
            {
                _disabledPacks.Add(packId);
                _log.LogInfo($"[ModPlatform] Pack '{packId}' disabled");
            }
            SaveDisabledPacks();
        }

        /// <summary>
        /// Saves the list of disabled packs to disk for persistence.
        /// </summary>
        private void SaveDisabledPacks()
        {
            try
            {
                string? packsDir = _packsDirectory?.Value;
                if (string.IsNullOrEmpty(packsDir)) return;
                string filePath = Path.Combine(packsDir, DisabledPacksFile);
                string json = JsonConvert.SerializeObject(_disabledPacks.ToList());
                File.WriteAllText(filePath, json, System.Text.Encoding.UTF8);
                _log.LogInfo($"[ModPlatform] Saved {_disabledPacks.Count} disabled pack(s) to {DisabledPacksFile}");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] Failed to save disabled packs: {ex}");
            }
        }

        /// <summary>
        /// Loads the list of disabled packs from disk.
        /// </summary>
        private void LoadDisabledPacks()
        {
            try
            {
                string? packsDir = _packsDirectory?.Value;
                if (string.IsNullOrEmpty(packsDir)) return;
                string filePath = Path.Combine(packsDir, DisabledPacksFile);
                if (!File.Exists(filePath)) return;
                string json = File.ReadAllText(filePath, Encoding.UTF8);
                List<string>? disabled = JsonConvert.DeserializeObject<List<string>>(json);
                if (disabled != null)
                {
                    _disabledPacks.Clear();
                    foreach (string packId in disabled)
                    {
                        _disabledPacks.Add(packId);
                    }
                    _log.LogInfo($"[ModPlatform] Loaded {_disabledPacks.Count} disabled pack(s) from {DisabledPacksFile}");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[ModPlatform] Failed to load disabled packs: {ex}");
            }
        }

        /// <summary>
        /// Shuts down the mod platform and disposes all resources.
        /// Call from <see cref="Plugin.OnDestroy"/>.
        /// </summary>
        /// <summary>
        /// Shuts down non-bridge resources (file watchers, HMR) while keeping the
        /// bridge server alive. Called when RuntimeDriver is destroyed by DINO's
        /// scene transitions — the bridge must survive for CLI/MCP tools to work.
        /// </summary>
        public void ShutdownNonBridge()
        {
            _log?.LogInfo("[ModPlatform] Partial shutdown (keeping bridge)...");

            try
            {
                if (_hotReloadBridge != null)
                {
                    _hotReloadBridge.OnRuntimeUpdated -= OnHotReloadCompleted;
                }

                if (_hotReloadBridge != null)
                {
                    _hotReloadBridge.Dispose();
                    _hotReloadBridge = null;
                }

                if (_packFileWatcher != null)
                {
                    _packFileWatcher.Dispose();
                    _packFileWatcher = null;
                }
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[ModPlatform] Error during partial shutdown: {ex}");
            }

            _log?.LogInfo("[ModPlatform] Partial shutdown complete. Bridge server still running.");
        }

        /// <summary>
        /// Full shutdown including bridge server. Only call on game exit.
        /// </summary>
        public void Shutdown()
        {
            _log?.LogInfo("[ModPlatform] Full shutdown...");

            try
            {
                if (_gameBridgeServer != null)
                {
                    // #793: Avoid disposing the singleton if a newer ModPlatform instance owns it.
                    if (object.ReferenceEquals(Plugin.SharedBridgeServer, _gameBridgeServer))
                    {
                        _gameBridgeServer?.Dispose();
                    }
                    else
                    {
                        _log?.LogDebug("[ModPlatform] Skipping bridge dispose — newer ModPlatform owns the singleton.");
                    }
                    _gameBridgeServer = null;
                }

                if (_hotReloadBridge != null)
                {
                    _hotReloadBridge.OnRuntimeUpdated -= OnHotReloadCompleted;
                }

                ShutdownNonBridge();
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[ModPlatform] Error during shutdown: {ex}");
            }

            _initialized = false;
            _worldReady = false;
            _log?.LogInfo("[ModPlatform] Shutdown complete.");
        }

        /// <summary>
        /// Disposes the platform by running the full shutdown path.
        /// </summary>
        public void Dispose()
        {
            Shutdown();
        }
    }
}
