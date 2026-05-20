#nullable enable
// Iter-144 #543 gray-freeze patch — pre-existing DF analyzer warnings in this file are
// outside the scope of the patch and tracked separately (see Pattern Catalog #105/#106/#111/#231).
#pragma warning disable DF0105 // event-lifecycle asymmetry (pre-existing, tracked)
#pragma warning disable DF0106 // implicit File.ReadAllText encoding (pre-existing, tracked)
#pragma warning disable DF0111 // empty catch block (pre-existing safe-swallows, tracked)
#pragma warning disable DF1006 // disposable field (pre-existing BepInEx-owned, tracked)
using System;
using System.IO;
using System.Threading;
using BepInEx;
using BepInEx.Configuration;
using BepInEx.Logging;
using DINOForge.Runtime.UI;
using DINOForge.SDK;
using HarmonyLib;
using Unity.Entities;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

namespace DINOForge.Runtime
{
    /// <summary>
    /// BepInEx entry point for the DINOForge mod platform.
    /// Bootstraps the <see cref="ModPlatform"/> orchestrator, registers ECS systems,
    /// and wires up UI overlays and hot reload.
    ///
    /// IMPORTANT: The BepInEx-managed GameObject (this.gameObject) gets destroyed
    /// during DINO's scene transitions, even with DontDestroyOnLoad. To survive,
    /// we create a separate "DINOForge_Root" GameObject with HideAndDontSave flags
    /// and attach all persistent MonoBehaviours to it. This matches the pattern
    /// used by devopsdinosaur/dno-mods where ECS systems outlive MonoBehaviours.
    /// </summary>
    [BepInPlugin(PluginInfo.GUID, PluginInfo.NAME, PluginInfo.VERSION)]
    public class Plugin : BaseUnityPlugin
    {
        private static ManualLogSource Log = null!;
        private Harmony? _harmony;

        // Static constructor fires BEFORE Awake — probe entry point
        static Plugin()
        {
            try
            {
                string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] [STATIC] Plugin class referenced\n");
            }
            catch { } // safe-swallow: diagnostic only
        }

        /// <summary>
        /// The persistent GameObject that survives scene changes.
        /// All UI and runtime components live here, NOT on the BepInEx-managed gameObject.
        /// </summary>
        internal static GameObject? PersistentRoot;

        // Captured at Awake for SceneManager resurrection callback
        private static ManualLogSource? _resurrectionLog;
        private static ConfigFile? _resurrectionConfig;
        private static bool _resurrectionDump;
        private static string _resurrectionDumpPath = "";

        /// <summary>Flag set by KeyInputSystem when F9 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF9Toggle;

        /// <summary>Flag set by KeyInputSystem when F10 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF10Toggle;

        /// <summary>Flag indicating PersistentRoot needs resurrection.</summary>
        internal static volatile bool NeedsResurrection;

        // Deferred TryResurrect: set by OnSceneLoaded or KeyInputSystem.OnCreate when a scene
        // transition or ECS world creation is detected. Checked by the background polling thread
        // which runs AFTER Plugin.Awake() completes. This prevents TryResurrect from racing
        // with Plugin.Awake() on a new RuntimeDriver.
        internal static volatile bool NeedsDeferredResurrection;
        internal static string? LastSceneNameForResurrection;

        /// <summary>
        /// Static singleton bridge server that survives RuntimeDriver destruction.
        /// Created once, thread owned by Plugin class (not by any MonoBehaviour).
        /// </summary>
        internal static Bridge.GameBridgeServer? SharedBridgeServer;

        private void Awake()
        {
            Log = Logger;
            Log.LogInfo("[DINOForge] Plugin.Awake() ENTRY");
            Log.LogInfo($"DINOForge Runtime v{PluginInfo.VERSION} loading...");

            // Config for debug features
            ConfigEntry<bool> dumpOnStartup = Config.Bind("Debug", "DumpOnStartup", true,
                "Automatically dump entity/component data when the game loads");
            ConfigEntry<string> dumpOutputPath = Config.Bind("Debug", "DumpOutputPath",
                Path.Combine(Paths.BepInExRootPath, "dinoforge_dumps"),
                "Directory to write entity/component dump files");

            // Detect game and log version compatibility info
            try
            {
                var bepinexVersion = typeof(BaseUnityPlugin).Assembly.GetName().Version?.ToString() ?? "unknown";
                Log.LogInfo($"DINOForge v{PluginInfo.VERSION} | BepInEx {bepinexVersion} | Unity {Application.unityVersion}");
                Log.LogInfo($"Platform: {Application.platform}");
                LogInstallDiagnostics();
            }
            catch (Exception ex)
            {
                Log.LogWarning($"Version detection failed: {ex}");
            }

            // ECS Type Discovery - log all available component types for diagnostics
            try
            {
                Bridge.EcsTypeDiscovery.DiscoverAndLog();
                Log.LogInfo("[Plugin] ECS type discovery complete - check dinoforge_debug.log for details");
            }
            catch (Exception ex)
            {
                Log.LogWarning($"[Plugin] ECS type discovery failed: {ex}");
            }

            // Harmony — apply patches from this assembly
            // ModsButtonTextPatch (UI/UiGridHarmonyPatch.cs) intercepts Text/TMP_Text setters
            // to prevent DINO's UiGrid from overwriting our repurposed Mods button label.
            try
            {
                _harmony = new Harmony(PluginInfo.GUID);
                Bridge.DestroyGuardPatch.Apply(_harmony);
                Bridge.ResourcesUnloadGuardPatch.Apply(_harmony);
                Bridge.AssetBundleUnloadGuardPatch.Apply(_harmony);
                Bridge.SceneUnloadGuardPatch.Apply(_harmony);
                Bridge.WorldDisposeGuardPatch.Apply(_harmony);
                UI.ModsButtonTextPatch.Apply(_harmony);
                Log.LogInfo("Harmony initialized and patches applied.");
            }
            catch (Exception ex)
            {
                Log.LogError($"Harmony init/patch failed: {ex}");
            }

            // Create a dedicated persistent GameObject that won't be destroyed.
            // The BepInEx-managed gameObject gets cleaned up during DINO's scene
            // transitions. A separate object with HideAndDontSave survives.
            try
            {
                PersistentRoot = new GameObject("DINOForge_Root");
                PersistentRoot.hideFlags = HideFlags.HideAndDontSave;
                UnityEngine.Object.DontDestroyOnLoad(PersistentRoot);
                Log.LogInfo("[Plugin] Persistent root GameObject created.");
            }
            catch (Exception ex)
            {
                Log.LogError($"[Plugin] Failed to create persistent root: {ex}");
                return;
            }

            // Add the runtime driver to the persistent root.
            // RuntimeDriver is a MonoBehaviour that handles Update()-based polling
            // for the ECS world and hosts all UI components.
            try
            {
                RuntimeDriver driver = PersistentRoot.AddComponent<RuntimeDriver>();
                driver.Initialize(Logger, Config, dumpOnStartup.Value, dumpOutputPath.Value);
                Log.LogInfo("[Plugin] RuntimeDriver initialized on persistent root.");
            }
            catch (Exception ex)
            {
                Log.LogError($"[Plugin] RuntimeDriver setup failed: {ex}");
            }

            // Capture state for static resurrection callback (kept for emergency use)
            _resurrectionLog = Logger;
            _resurrectionConfig = Config;
            _resurrectionDump = dumpOnStartup.Value;
            _resurrectionDumpPath = dumpOutputPath.Value;

            StartResurrectionWatcher();

            WriteDebug("Awake completed");
            Log.LogInfo("DINOForge Runtime loaded successfully.");
            Log.LogInfo("[DINOForge] Plugin.Awake() EXIT");
        }

        /// <summary>
        /// Iter-144 #543/#546 gray-freeze fix: subscribe to <c>SceneManager.activeSceneChanged</c>
        /// rather than <c>sceneLoaded</c>. Per project_dino_runtime_execution_model.md (confirmed
        /// 2026-03-21), DINO replaces Unity's PlayerLoop entirely; <c>sceneLoaded</c> fires
        /// inconsistently (in-game probe iter-144 confirmed NO sceneLoaded post-OnDestroy) while
        /// <c>activeSceneChanged</c> reliably fires on the main thread for each scene transition.
        ///
        /// Also starts a Win32 background polling thread that calls TryResurrect on a grace-window
        /// timer in case NO scene-change event fires within the window (defense-in-depth, survives
        /// RuntimeDriver destruction since it lives on the Plugin class, not the MonoBehaviour).
        /// </summary>
        private static void StartResurrectionWatcher()
        {
            SceneManager.activeSceneChanged += OnActiveSceneChanged;
            WriteDebug("[Plugin] activeSceneChanged watcher registered (iter-144 #546 fix).");
            StartResurrectionFallbackThread();
        }

        private static void OnActiveSceneChanged(Scene oldScene, Scene newScene)
        {
            WriteDebug($"[Plugin] OnActiveSceneChanged: old='{oldScene.name}' new='{newScene.name}'");
            try
            {
                Bridge.KeyInputSystem.RecreateInCurrentWorld();
            }
            catch (Exception ex)
            {
                WriteDebug($"[Plugin] OnActiveSceneChanged RecreateInCurrentWorld failed: {ex.Message}");
            }
            // RuntimeDriver may have been destroyed when DINO destroyed our root.
            // Trigger resurrection here. IMPORTANT: we defer TryResurrect to the resurrection thread
            // (or the new RuntimeDriver's BG poll thread) instead of calling directly, since a brand
            // new RuntimeDriver may not have completed Initialize() yet at this exact tick.
            if (NeedsResurrection || PersistentRoot == null)
            {
                WriteDebug($"[Plugin] OnActiveSceneChanged: resurrection needed - NeedsRes={NeedsResurrection} rootNull={PersistentRoot == null}");
                LastSceneNameForResurrection = newScene.name;
                NeedsDeferredResurrection = true;
            }
        }

        // Iter-144 #546 fallback: Win32 background thread independent of any MonoBehaviour.
        // Survives RuntimeDriver destruction (the MB-owned background poll thread dies with its host).
        // Polls NeedsResurrection every 500ms; if set and no scene event has cleared it within the
        // grace window, attempts TryResurrect directly. Plugin class is referenced as long as the
        // BepInEx assembly is loaded, so this thread persists across scene transitions.
        private static Thread? _resurrectionFallbackThread;
        private static volatile bool _resurrectionFallbackStop;

        private static void StartResurrectionFallbackThread()
        {
            if (_resurrectionFallbackThread != null) return;
            _resurrectionFallbackThread = new Thread(ResurrectionFallbackLoop)
            {
                Name = "DINOForge.ResurrectionFallback",
                IsBackground = true,
            };
            _resurrectionFallbackThread.Start();
            WriteDebug("[Plugin] Resurrection fallback thread started.");
        }

        private static void ResurrectionFallbackLoop()
        {
            DateTime lastNeedsObservedUtc = DateTime.MinValue;
            long iterationCount = 0;
            const int PollIntervalMs = 500;
            const int GraceWindowMs = 4000; // 4s after NeedsResurrection observed, attempt direct revive
            // Iter-144 #547 H6: 4-iter (2s) heartbeat — frequent enough to distinguish "Mono wedged"
            // from "no scene events firing yet" in the post-OnDestroy gray-freeze window. Previous 10s
            // cadence left ambiguous gaps where probe timing missed the window entirely.
            const int HeartbeatEveryNIterations = 4;
            WriteDebug("[Plugin] ResurrectionFallback: loop entered.");
            while (!_resurrectionFallbackStop)
            {
                try
                {
                    Thread.Sleep(PollIntervalMs);
                    iterationCount++;
                    // Iter-144 #547 H5: emit periodic heartbeat to prove Mono runtime + this thread are alive.
                    // If the gray-freeze is a native deadlock at runtime level, heartbeats stop appearing
                    // immediately after OnDestroy. If they keep appearing, the hang is elsewhere.
                    if (iterationCount % HeartbeatEveryNIterations == 0)
                    {
                        WriteDebug($"[Plugin] ResurrectionFallback heartbeat #{iterationCount} NeedsRes={NeedsResurrection} NeedsDefRes={NeedsDeferredResurrection} rootNull={PersistentRoot == null}");
                    }
                    if (!NeedsResurrection && !NeedsDeferredResurrection)
                    {
                        lastNeedsObservedUtc = DateTime.MinValue;
                        continue;
                    }
                    if (lastNeedsObservedUtc == DateTime.MinValue)
                    {
                        lastNeedsObservedUtc = DateTime.UtcNow;
                        WriteDebug("[Plugin] ResurrectionFallback: NeedsResurrection observed, starting grace timer.");
                        continue;
                    }
                    TimeSpan since = DateTime.UtcNow - lastNeedsObservedUtc;
                    if (since.TotalMilliseconds < GraceWindowMs) continue;
                    // Grace window exceeded with no scene-event resolution: attempt direct resurrect.
                    if (_resurrectionLog == null || _resurrectionConfig == null)
                    {
                        // Plugin.Awake never completed; can't resurrect. Reset timer to retry later.
                        WriteDebug("[Plugin] ResurrectionFallback: cannot revive (Plugin.Awake state not captured). Will retry.");
                        lastNeedsObservedUtc = DateTime.UtcNow;
                        continue;
                    }
                    string sceneName = LastSceneNameForResurrection ?? "fallback-unknown";
                    WriteDebug($"[Plugin] ResurrectionFallback: grace window {GraceWindowMs}ms exceeded — invoking TryResurrect (scene='{sceneName}').");
                    try
                    {
                        TryResurrect(sceneName, "ResurrectionFallbackThread");
                        // After attempt, clear flags so we don't spin; if revive failed, scene event/poller will re-set them.
                        NeedsResurrection = false;
                        NeedsDeferredResurrection = false;
                        lastNeedsObservedUtc = DateTime.MinValue;
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"[Plugin] ResurrectionFallback TryResurrect threw: {ex.Message}");
                        lastNeedsObservedUtc = DateTime.UtcNow; // back off, retry next grace window
                    }
                }
                catch (ThreadAbortException)
                {
                    break;
                }
                catch (Exception ex)
                {
                    WriteDebug($"[Plugin] ResurrectionFallback loop error: {ex.Message}");
                }
            }
            WriteDebug("[Plugin] Resurrection fallback thread exiting.");
        }

        /// <summary>
        /// Marks that TryResurrect should be called from the background polling thread.
        /// Called by KeyInputSystem.OnCreate when the ECS world is created during scene transition.
        /// The background thread will call TryResurrect after Plugin.Awake() has completed,
        /// ensuring resurrection parameters are available.
        /// </summary>
        internal static void MarkNeedsDeferredResurrection(string trigger)
        {
            if (NeedsDeferredResurrection) return; // Already set
            WriteDebug($"[Plugin] MarkNeedsDeferredResurrection via {trigger}");
            NeedsDeferredResurrection = true;
        }

        internal static void TryResurrect(string sceneName, string trigger)
        {
            // If PersistentRoot exists, RuntimeDriver is already running. But check if it was
            // initialized — if not (Plugin.Awake() crashed before completing), initialize it.
            if (PersistentRoot != null)
            {
                // Check if RuntimeDriver component exists and is initialized
                RuntimeDriver? existing = PersistentRoot.GetComponent<RuntimeDriver>();
                if (existing != null && existing.IsInitialized)
                {
                    WriteDebug($"[Plugin] TryResurrect ({trigger}): RuntimeDriver already running, ensuring KeyInputSystem is registered...");
                    // CRITICAL: Always ensure KeyInputSystem is registered in the current world,
                    // even if RuntimeDriver is already initialized. Scene transitions may have
                    // created a new world that KeyInputSystem needs to be registered in.
                    Bridge.KeyInputSystem.RecreateInCurrentWorld();
                    return;
                }
                // RuntimeDriver exists but wasn't initialized — initialize it
                if (existing != null)
                {
                    WriteDebug($"[Plugin] TryResurrect ({trigger}): RuntimeDriver exists but not initialized, initializing...");
                    existing.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);
                    return;
                }
                // No RuntimeDriver component — create one
                WriteDebug($"[Plugin] TryResurrect ({trigger}): PersistentRoot exists but no RuntimeDriver, adding component...");
                RuntimeDriver driver = PersistentRoot.AddComponent<RuntimeDriver>();
                driver.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);
                return;
            }

            WriteDebug($"[Plugin] PersistentRoot null via {trigger} on '{sceneName}' — resurrecting...");
            try
            {
                // Try to attach RuntimeDriver to DINO's main camera — DINO never destroys its own camera
                Camera? cam = Camera.main ?? (Camera.allCameras.Length > 0 ? Camera.allCameras[0] : null);
                GameObject host;
                if (cam != null)
                {
                    host = cam.gameObject;
                    WriteDebug($"[Plugin] Attaching to existing camera '{host.name}'");
                }
                else
                {
                    // Fallback: create our own object
                    host = new GameObject("DINOForge_Root");
                    host.hideFlags = HideFlags.HideAndDontSave;
                    UnityEngine.Object.DontDestroyOnLoad(host);
                    WriteDebug($"[Plugin] No camera found, using new GameObject");
                }
                PersistentRoot = host;

                RuntimeDriver driver = host.AddComponent<RuntimeDriver>();
                driver.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);

                // Immediately register KeyInputSystem in the current ECS world.
                // The polling thread will also do this, but scene transitions may have already
                // created a new DefaultGameObjectInjectionWorld that the thread hasn't caught yet.
                // This call bridges the gap so the pump is active without waiting for a poll cycle.
                Bridge.KeyInputSystem.RecreateInCurrentWorld();
                WriteDebug($"[Plugin] Resurrection complete via {trigger} on '{sceneName}' host='{host.name}'.");
            }
            catch (Exception ex)
            {
                WriteDebug($"[Plugin] Resurrection FAILED via {trigger}: {ex.Message}");
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }

        private static void LogInstallDiagnostics()
        {
            string loadedAssemblyPath = typeof(Plugin).Assembly.Location;
            string primaryRuntimePath = Path.Combine(Paths.PluginPath, "DINOForge.Runtime.dll");
            string legacyRuntimePath = Path.Combine(Paths.BepInExRootPath, "ecs_plugins", "DINOForge.Runtime.dll");
            string backupRuntimePath = Path.Combine(Paths.PluginPath, "DINOForge.Runtime.dll.bak");

            Log.LogInfo($"[Plugin] Loaded runtime assembly from: {loadedAssemblyPath}");
            WriteDebug($"[Plugin] Loaded runtime assembly from: {loadedAssemblyPath}");

            if (File.Exists(legacyRuntimePath))
            {
                string message = $"[Plugin] Legacy runtime copy detected at deprecated path: {legacyRuntimePath}";
                Log.LogWarning(message);
                WriteDebug(message);
            }

            if (File.Exists(primaryRuntimePath) && File.Exists(legacyRuntimePath))
            {
                string message = $"[Plugin] Duplicate runtime assemblies detected. Primary='{primaryRuntimePath}', Legacy='{legacyRuntimePath}'";
                Log.LogWarning(message);
                WriteDebug(message);
            }

            if (File.Exists(backupRuntimePath))
            {
                string message = $"[Plugin] Stale runtime backup file detected: {backupRuntimePath}";
                Log.LogWarning(message);
                WriteDebug(message);
            }

            if (!string.Equals(loadedAssemblyPath, primaryRuntimePath, StringComparison.OrdinalIgnoreCase))
            {
                string message = $"[Plugin] Runtime loaded from non-canonical location. Expected '{primaryRuntimePath}', actual '{loadedAssemblyPath}'";
                Log.LogWarning(message);
                WriteDebug(message);
            }
        }

        private void OnDestroy()
        {
            // The BepInEx-managed object is being destroyed (expected in DINO).
            // The persistent root and RuntimeDriver continue running independently.
            Log?.LogInfo("[Plugin] BepInEx plugin object OnDestroy (persistent root still alive).");
            try { _harmony?.UnpatchSelf(); } catch (Exception ex) { WriteDebug($"OnDestroy Harmony.UnpatchSelf failed: {ex.Message}"); }
            // Iter-144 #547 H5 gray-freeze fix: do NOT unsubscribe activeSceneChanged here.
            // The handler is a static method on the Plugin class; the static delegate survives
            // BepInEx Plugin instance destruction. Previously we unsubscribed here, breaking
            // resurrection on second-and-later scene transitions (only the Win32 fallback thread
            // could revive). Keeping the subscription live is the correct behavior — there's
            // no leak because the target is a static method.
            // Harmony unpatch is also deliberately skipped — runtime patches must persist across
            // BepInEx Plugin object death since the actual functionality lives on RuntimeDriver/
            // ModPlatform which outlive this BepInEx wrapper.
            WriteDebug("OnDestroy called (BepInEx object only); activeSceneChanged + fallback thread persist by design (iter-144 #547).");
        }
    }

    /// <summary>
    /// Persistent MonoBehaviour that runs on the DINOForge_Root GameObject.
    /// Uses Update()-based polling instead of coroutines to detect the ECS world,
    /// since coroutines die with their host MonoBehaviour and the BepInEx object
    /// gets destroyed before the ECS world is ready.
    ///
    /// Hosts all UI components (debug overlay on F9, mod menu on F10).
    ///
    /// Key design: F9/F10 handling lives HERE, not in DFCanvas or ModMenuOverlay,
    /// so the shortcuts always work regardless of which UI layer is active.
    /// </summary>
    internal class RuntimeDriver : MonoBehaviour
    {
        private ManualLogSource _log = null!;
        private ConfigFile _config = null!;
        private bool _dumpOnStartup;
        private string _dumpOutputPath = "";


        private ModPlatform? _modPlatform;

        // UGUI system (preferred). Null if UGUI setup failed.
        internal DFCanvas? _dfCanvas;

        // Active UI hosts.
        // _modMenuHost is always set to the active menu (UGUI when healthy, IMGUI fallback otherwise).
        // _debugOverlay is ALWAYS added (it owns the IMGUI F9 debug panel).
        private IModMenuHost? _modMenuHost;
        private IModSettingsHost? _modSettingsHost;
        private DebugOverlayBehaviour? _debugOverlay;
        private HudIndicator? _hudIndicator;
        private NativeMenuInjector? _nativeMenuInjector;

        // _uguiReady: true once DFCanvas.Start() reports success via IsReady.
        // We check this each Update() because DFCanvas.Start() runs after Initialize().
        internal bool _uguiReady;
        // _uguiChecked: we only need to check DFCanvas readiness once after it has
        // had at least one frame to run its Start().
        internal bool _uguiChecked;

        /// <summary>
        /// Registers KeyInputSystem in the given ECS world if not already registered.
        /// Called every poll cycle to ensure the pump survives scene transitions.
        /// Safe to call multiple times (GetOrCreateSystem is idempotent).
        /// </summary>
        private void TryRegisterKeyInputSystem(World world)
        {
            if (_registeredWorldInstance != null && ReferenceEquals(_registeredWorldInstance, world)) return;
            try
            {
                world.GetOrCreateSystem<Bridge.KeyInputSystem>();
                _log.LogInfo($"[RuntimeDriver] KeyInputSystem registered in world '{world.Name}'.");
                _registeredWorldInstance = world;
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] TryRegisterKeyInputSystem failed: {ex}");
            }
        }

        private bool _worldFound;
        private bool _initialized;

        /// <summary>Public accessor for TryResurrect to check if RuntimeDriver is initialized.</summary>
        internal bool IsInitialized => _initialized;
        private bool _catalogRebuilt;
        private float _worldPollTimer;
        // Tracks the ECS world instance that KeyInputSystem was registered in.
        // When DINO transitions scenes, it destroys the old world and creates a new one.
        // We detect this by comparing the current DefaultGameObjectInjectionWorld against
        // _registeredWorldInstance and re-registering KeyInputSystem in the new world.
        private World? _registeredWorldInstance;
        // Cross-thread flag: true once OnDestroy is called. The background polling thread
        // checks this to avoid calling OnWorldReady after the RuntimeDriver is destroyed.
        private volatile bool _destroyed;
        private readonly ManualResetEventSlim _backgroundPollStopEvent = new(false);

        // Iter-144 #543 gray-freeze fix: cross-thread static flag observable by any subsystem
        // (e.g. VanillaCatalog.Build, ContentLoader pack registration) so they can short-circuit
        // cleanly when DINO is tearing down the ECS world. Set true at the TOP of OnDestroy
        // before any other shutdown work, so the window between scene-transition begin and our
        // OnDestroy completion is observable to callers running on the main thread.
        private static volatile bool s_isBeingDestroyed;
        public static bool IsBeingDestroyed => s_isBeingDestroyed;

        /// <summary>Polling interval in seconds for ECS world detection.</summary>
        private const float WorldPollInterval = 0.5f;

        /// <summary>
        /// Initializes the driver with config and logger references.
        /// Called immediately after AddComponent by Plugin.Awake().
        /// </summary>
        public void Initialize(ManualLogSource log, ConfigFile config, bool dumpOnStartup, string dumpOutputPath)
        {
            _log = log;
            _config = config;
            _dumpOnStartup = dumpOnStartup;
            _dumpOutputPath = dumpOutputPath;
            _initialized = true;
            _log.LogInfo("[DINOForge] RuntimeDriver.Initialize() ENTRY");

            CleanupUiInterceptors();

            // Initialize Kenney CC0 UI asset loader.
            // Sprites are expected at BepInEx/plugins/dinoforge-ui-assets/ (deployed by MSBuild target).
            // If the directory or files are absent UiAssets falls back silently — all properties return null.
            try
            {
                UiAssets.Initialize(BepInEx.Paths.PluginPath);
                if (UiAssets.MissingFiles.Count > 0)
                {
                    _log.LogInfo($"[RuntimeDriver] UiAssets: {UiAssets.MissingFiles.Count} sprite(s) not found " +
                        $"— flat-colour fallback active. See src/Runtime/UI/Assets/README.md for download instructions.");
                }
                else
                {
                    _log.LogInfo("[RuntimeDriver] UiAssets: sprites loaded from disk.");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] UiAssets initialization failed: {ex}");
            }

            // Initialize ModPlatform orchestrator
            try
            {
                _modPlatform = new ModPlatform();
                _modPlatform.Initialize(_log, _config, gameObject);
                _log.LogInfo("[RuntimeDriver] ModPlatform initialized.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[RuntimeDriver] ModPlatform initialization failed: {ex}");
                _modPlatform = null;
            }

            // Add MainThreadDispatcher for IPC bridge support
            try
            {
                gameObject.AddComponent<Bridge.MainThreadDispatcher>();
                _log.LogInfo("[RuntimeDriver] Added MainThreadDispatcher.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[RuntimeDriver] MainThreadDispatcher setup failed: {ex}");
            }

            // ── Step 1: Always add DebugOverlayBehaviour ────────────────────────────
            // This component owns the IMGUI F9 debug panel and must always be present
            // so F9 works even when UGUI is active or fails.  DFCanvas also shows a
            // UGUI debug panel (DebugPanel) when healthy, but DebugOverlayBehaviour
            // is the guaranteed fallback.
            try
            {
                _debugOverlay = gameObject.AddComponent<DebugOverlayBehaviour>();
                _log.LogInfo("[RuntimeDriver] Added DebugOverlayBehaviour (guaranteed F9 handler).");
            }
            catch (Exception ex)
            {
                _log.LogError($"[RuntimeDriver] DebugOverlayBehaviour setup failed: {ex}");
            }

            // ── KeyInputSystem ECS callbacks (DISABLED) ────────────────────────────────
            // ECS callbacks are the reliable toggle path — KeyInputSystem.OnUpdate runs
            // in the ECS loop and correctly sees both physical and synthetic key presses.
            // The background thread's GetAsyncKeyState DOES NOT reliably see synthetic
            // keybd_event input from external processes, so ECS callbacks are preferred.
            // Background thread F9/F10 polling is disabled to prevent double-toggles.
            Bridge.KeyInputSystem.OnF9Pressed = () =>
            {
                try
                {
                    WriteDebug("[RuntimeDriver] F9 pressed (via KeyInputSystem)");
                    if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleDebug();
                    else _debugOverlay?.Toggle();
                }
                catch (Exception ex)
                {
                    WriteDebug($"[RuntimeDriver] F9 toggle failed: {ex.GetType().Name} - {ex.Message}");
                }
            };
            Bridge.KeyInputSystem.OnF10Pressed = () =>
            {
                try
                {
                    WriteDebug("[RuntimeDriver] F10 pressed (via KeyInputSystem)");
                    if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
                    else _modMenuHost?.Toggle();
                }
                catch (Exception ex)
                {
                    WriteDebug($"[RuntimeDriver] F10 toggle failed: {ex.GetType().Name} - {ex.Message}");
                }
            };

            // ── Wire HMR pack reload callback (can be invoked from background thread) ──
            Bridge.KeyInputSystem.OnPackReloadRequested = () =>
            {
                try
                {
                    WriteDebug("[RuntimeDriver] Pack reload requested (via OnPackReloadRequested)");
                    if (_modPlatform != null)
                    {
                        _modPlatform.LoadPacks();
                        _log?.LogInfo("[RuntimeDriver] Packs reloaded via HMR.");
                    }
                }
                catch (Exception ex)
                {
                    _log?.LogWarning($"[RuntimeDriver] Pack reload failed: {ex}");
                }
            };

            // ── Step 2: Attempt UGUI canvas setup ───────────────────────────────────
            // DFCanvas.Initialize() builds the canvas hierarchy synchronously and calls
            // OnInitSuccess immediately if successful, or OnInitFailed if it throws.
            // We register both callbacks so that _uguiReady is set on the main thread,
            // not from the background polling thread (which would cause UnityException).
            bool uguiAddedOk = false;
            try
            {
                _dfCanvas = gameObject.AddComponent<DFCanvas>();

                // Register callbacks BEFORE Initialize() — Initialize() calls them synchronously.
                _dfCanvas.OnInitSuccess = () =>
                {
                    _uguiReady = true;
                    _uguiChecked = true;
                    _log.LogInfo("[RuntimeDriver] DFCanvas.OnInitSuccess — UGUI canvas ready on main thread.");
                    WriteDebug("[RuntimeDriver] DFCanvas.OnInitSuccess: UGUI is ready.");
                };
                _dfCanvas.OnInitFailed = () =>
                {
                    _log.LogWarning("[RuntimeDriver] DFCanvas.OnInitFailed — activating IMGUI fallback.");
                    _uguiReady = false;
                    _uguiChecked = true;
                    ActivateImguiFallback();
                };

                _dfCanvas.Initialize(_log);

                uguiAddedOk = true;
                _log.LogInfo("[RuntimeDriver] Added DFCanvas — UGUI canvas built in Initialize().");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] DFCanvas AddComponent failed, falling back to IMGUI immediately: {ex}");

                if (_dfCanvas != null)
                {
                    Destroy(_dfCanvas);
                    _dfCanvas = null;
                }
            }

            if (!uguiAddedOk)
            {
                // UGUI component could not even be added — activate IMGUI now.
                _uguiChecked = true;
                ActivateImguiFallback();
            }

            // ── Step 3: Add NativeMenuInjector for main menu button injection ──────
            // This component monitors scene changes and injects a "Mods" button into
            // the native game menus (main menu, pause menu) next to Settings/Options.
            try
            {
                _nativeMenuInjector = gameObject.AddComponent<NativeMenuInjector>();
                _nativeMenuInjector.SetLogger(_log);
                // We'll wire the overlay reference later once it's created
                _log.LogInfo("[RuntimeDriver] Added NativeMenuInjector — will inject Mods button into native menus.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] NativeMenuInjector setup failed: {ex}");
            }

            // ── Step 3b: UiEventInterceptor intentionally disabled ──
            // Interceptor diagnostics mutate button object names and can interfere with
            // NativeMenuInjector idempotency and click routing in production runtime.
            _log.LogInfo("[RuntimeDriver] UiEventInterceptor disabled for native menu stability.");

            // ── Step 4: Start HMR (Hot Module Reload) signal watcher ─────────────
            // Watches for DINOForge_HotReload signal file in BepInEx root
            // When detected, triggers soft UI + pack reload without full game restart
            StartHmrWatcher();

            // ── Step 5: Start background polling (ECS world, catalog rebuild, heartbeats) ──
            // MonoBehaviour.Update() NEVER fires in DINO — background thread polling is required.
            StartBackgroundPollingThread();

            // ── Step 6: Log key handler registration ────────────────────────────────
            WriteDebug($"[RuntimeDriver.Initialize] ENTRY — Initialize starting on {gameObject.name}");
            _log.LogInfo($"[RuntimeDriver] F9/F10 key handlers registered on {gameObject.name}.");
            _log.LogInfo("[RuntimeDriver] Waiting for ECS World (Update polling)...");
            _log.LogInfo("[DINOForge] RuntimeDriver.Initialize() EXIT");
        }

        /// <summary>
        /// Starts a background thread that monitors for the DINOForge_HotReload signal file.
        /// When the file is detected, invokes reload directly from the background thread.
        ///
        /// CRITICAL: MonoBehaviour.Update() NEVER fires in DINO (scene transitions destroy it).
        /// We invoke reload methods directly from this background thread, using the same pattern
        /// as F9/F10 which work via KeyInputSystem callbacks from background thread input polling.
        ///
        /// Direct thread calls work in Mono 2021.3 on DontDestroyOnLoad objects.
        /// </summary>
        private void StartHmrWatcher()
        {
            System.Threading.ThreadPool.QueueUserWorkItem(_ =>
            {
                try
                {
                    string signalPath = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "DINOForge_HotReload");
                    while (true)
                    {
                        System.Threading.Thread.Sleep(2000);
                        if (System.IO.File.Exists(signalPath))
                        {
                            try { System.IO.File.Delete(signalPath); } catch { } // safe-swallow: HMR signal file cleanup, non-critical

                            // Direct invocation from background thread — works in Mono 2021.3
                            // Same pattern as F9/F10 key polling (no Update() required)
                            _log?.LogInfo("[RuntimeDriver] HMR: Signal detected, reloading packs...");

                            try
                            {
                                // Invoke pack reload via KeyInputSystem callback (works from background thread)
                                Bridge.KeyInputSystem.OnPackReloadRequested?.Invoke();
                            }
                            catch (System.Exception ex)
                            {
                                _log?.LogWarning($"[RuntimeDriver] HMR: Pack reload invocation failed: {ex}");
                            }

                            // Re-initialize UGUI if it exists
                            try
                            {
                                RuntimeDriver? driver = Plugin.PersistentRoot?.GetComponent<RuntimeDriver>();
                                if (driver != null)
                                {
                                    // Reset UGUI state flags so on-next-Update it rebuilds
                                    driver._uguiReady = false;
                                    driver._uguiChecked = false;
                                    driver._dfCanvas = null;
                                    _log?.LogInfo("[RuntimeDriver] HMR: UGUI state reset for rebuild.");
                                }
                            }
                            catch (System.Exception ex)
                            {
                                _log?.LogWarning($"[RuntimeDriver] HMR: UGUI reset failed: {ex}");
                            }

                            _log?.LogInfo("[RuntimeDriver] HMR: Reload complete.");
                        }
                    }
                }
                catch { } // safe-swallow: HMR reload best-effort, non-critical
            });
        }

        /// <summary>
        /// Starts a background thread that handles all polling previously done in Update().
        /// MonoBehaviour.Update() NEVER fires in DINO, so we run:
        ///   - F9/F10 key polling via Win32 GetAsyncKeyState (works from background thread)
        ///   - UGUI canvas readiness checks
        ///   - ECS World availability polling
        ///   - VanillaCatalog rebuild once world is fully loaded
        ///   - Heartbeat logging
        ///
        /// Uses UnityEngine.Object.FindObjectsOfType (NOT FindObjectsOfTypeAll) to avoid
        /// deadlock during asset loading in Mono 2021.3.
        /// </summary>
        private void StartBackgroundPollingThread()
        {
            System.Threading.ThreadPool.QueueUserWorkItem(_ =>
            {
                try
                {
                    int heartbeatCounter = 0;
                    while (true)
                    {
                        // sync-over-async-unavoidable: background thread control signal (50ms timeout, no deadlock)
                        if (_backgroundPollStopEvent.Wait(50))  // Signaled = destroyed
                            break;

                        // Guard: only run if initialized
                        if (!_initialized) continue;

                        // Heartbeat logging (every 1 sec for first 10, then every 10 sec)
                        heartbeatCounter++;
                        bool earlyHeartbeat = heartbeatCounter <= 200; // ~10 seconds at 50ms interval
                        bool laterHeartbeat = heartbeatCounter % 200 == 0; // Every 10 seconds
                        if (earlyHeartbeat || laterHeartbeat)
                        {
                            _log?.LogDebug($"[RuntimeDriver] Background poll heartbeat #{heartbeatCounter} worldFound={_worldFound}");
                        }

                        // ── Deferred TryResurrect ───────────────────────────────────
                        // If OnSceneLoaded or KeyInputSystem.OnCreate set NeedsDeferredResurrection,
                        // call TryResurrect now. The background thread runs AFTER Plugin.Awake() completes,
                        // so TryResurrect will succeed (Plugin.Awake() sets _resurrectionLog and _resurrectionConfig).
                        if (Plugin.NeedsDeferredResurrection)
                        {
                            Plugin.NeedsDeferredResurrection = false;
                            try
                            {
                                WriteDebug("[RuntimeDriver] Background poll: calling TryResurrect (deferred)");
                                Plugin.TryResurrect(Plugin.LastSceneNameForResurrection ?? "unknown", "BackgroundPoll_Deferred");
                            }
                            catch (Exception ex)
                            {
                                WriteDebug($"[RuntimeDriver] Deferred TryResurrect failed: {ex.Message}");
                            }
                        }

                        // ── F9/F10 key polling DISABLED ───────────────────────────────
                        // F9/F10 are now handled exclusively by KeyInputSystem ECS callbacks
                        // (OnF9Pressed/OnF10Pressed) which reliably see both physical and
                        // synthetic key presses. GetAsyncKeyState from this background thread
                        // does NOT reliably see synthetic keybd_event from external processes.
                        // Background polling caused double-toggles when both paths were active.
                        //
                        // F10 background thread DEAD CODE (kept for reference):
                        if (false) // DISABLED
                        {
                            System.Threading.Thread.Sleep(50); // Debounce
                            if (false)
                            {
                                try
                                {
                                    _log?.LogDebug("[RuntimeDriver] F10 pressed (background thread)");
                                    if (_uguiReady && _dfCanvas != null)
                                    {
                                        _dfCanvas.ToggleModMenu();
                                    }
                                    else if (_modMenuHost != null)
                                    {
                                        _modMenuHost.Toggle();
                                    }
                                }
                                catch (System.Exception ex)
                                {
                                    _log?.LogWarning($"[RuntimeDriver] F10 toggle failed: {ex}");
                                }

                                // Wait for key release (dead code)
                                System.Threading.Thread.Sleep(50);
                            }
                        }

                        // ── DFCanvas readiness is handled by OnInitSuccess callback ──────────────
                        // No need to poll IsReady from background thread (causes UnityException).
                        // The callback is invoked synchronously from DFCanvas.Initialize() on main thread.

                        // ── ECS World polling ────────────────────────────────────────────
                        if (!_worldFound)
                        {
                            // Bail out if RuntimeDriver was destroyed (e.g., during scene transition).
                            // OnDestroy sets _destroyed=true so the background thread exits cleanly.
                            if (_destroyed) break;

                            _worldPollTimer += 0.05f; // Add 50ms per poll iteration
                            if (_worldPollTimer >= WorldPollInterval)
                            {
                                _worldPollTimer = 0f;
                                try
                                {
                                    World? world = World.DefaultGameObjectInjectionWorld;
                                    if (world != null && world.IsCreated)
                                    {
                                        // Register KeyInputSystem immediately — ECS systems survive scene transitions.
                                        // This ensures the main-thread pump (DrainQueue) is active even during InitialGameLoader.
                                        TryRegisterKeyInputSystem(world);

                                        Scene activeScene = UnityEngine.SceneManagement.SceneManager.GetActiveScene();
                                        bool isLoaderScene = activeScene.name != null &&
                                            activeScene.name.IndexOf("InitialGameLoader", StringComparison.OrdinalIgnoreCase) >= 0;
                                        if (isLoaderScene)
                                        {
                                            _log?.LogDebug("[RuntimeDriver] ECS world found but at InitialGameLoader — waiting for scene transition.");
                                            continue; // Skip pack loading; NativeMenuInjector will trigger LoadScene(1)
                                        }

                                        _worldFound = true;
                                        OnWorldReady(world);
                                    }
                                }
                                catch
                                {
                                    // World not ready yet, will retry next poll
                                }
                            }
                        }
                        // World found — now check if we need to rebuild the catalog
                        else if (!_catalogRebuilt)
                        {
                            if (_destroyed) break;
                            // Also handle world changes (scene transitions): re-register KeyInputSystem
                            // in the new DefaultGameObjectInjectionWorld if it changed since last registration.
                            try
                            {
                                World? w = World.DefaultGameObjectInjectionWorld;
                                if (w != null && w.IsCreated && (_registeredWorldInstance == null || !ReferenceEquals(_registeredWorldInstance, w)))
                                {
                                    TryRegisterKeyInputSystem(w);
                                }
                            }
                            catch { } // safe-swallow: ECS world discovery best-effort

                            // Catalog rebuild: only trigger once when enough entities exist
                            try
                            {
                                World? w2 = World.DefaultGameObjectInjectionWorld;
                                if (w2 != null && w2.IsCreated)
                                {
                                    int entityCount = w2.EntityManager.UniversalQuery.CalculateEntityCount();
                                    if (entityCount > 1000)
                                    {
                                        _catalogRebuilt = true;
                                        _log?.LogInfo($"[RuntimeDriver] Catalog rebuild triggered ({entityCount} entities)");
                                        _modPlatform?.RebuildCatalogAndApplyStats(w2);
                                    }
                                }
                            }
                            catch { } // safe-swallow: catalog rebuild best-effort
                        }
                        // Stable state: detect world changes (scene transitions) and re-register KeyInputSystem.
                        // After scene transitions, DINO creates a new ECS world and updates
                        // DefaultGameObjectInjectionWorld. We detect this and re-register KeyInputSystem
                        // so DrainQueue keeps pumping — this unblocks the MCP bridge.
                        else
                        {
                            if (_destroyed) break;
                            try
                            {
                                World? current = World.DefaultGameObjectInjectionWorld;
                                if (current != null && current.IsCreated && !ReferenceEquals(current, _registeredWorldInstance))
                                {
                                    _registeredWorldInstance = current;
                                    _log?.LogInfo($"[RuntimeDriver] ECS world changed to '{current.Name}' — re-registering KeyInputSystem");
                                    try
                                    {
                                        current.GetOrCreateSystem<Bridge.KeyInputSystem>();
                                        _log?.LogInfo("[RuntimeDriver] KeyInputSystem re-registered in new world.");
                                    }
                                    catch (Exception ex)
                                    {
                                        _log?.LogWarning($"[RuntimeDriver] KeyInputSystem re-registration failed: {ex}");
                                    }
                                }
                            }
                            catch { } // safe-swallow: Key system discovery best-effort
                        }
                    }
                }
                catch (System.Exception ex)
                {
                    _log?.LogError($"[RuntimeDriver] Background polling thread exception: {ex}");
                }
            });
        }

        /// <summary>
        /// Win32 API: GetAsyncKeyState - polls keyboard state without blocking.
        /// Returns a short where bit 15 (0x8000) indicates key is currently pressed.
        /// </summary>
        [System.Runtime.InteropServices.DllImport("user32.dll", SetLastError = true)]
        private static extern short GetAsyncKeyState(int vKey);

        private void CleanupUiInterceptors()
        {
            try
            {
                UiEventInterceptor[] interceptors = Resources.FindObjectsOfTypeAll<UiEventInterceptor>();
                foreach (UiEventInterceptor interceptor in interceptors)
                {
                    if (interceptor == null) continue;
                    _log.LogWarning($"[RuntimeDriver] Destroying stale UiEventInterceptor on '{interceptor.gameObject.name}'.");
                    Destroy(interceptor);
                }

                Button[] buttons = Resources.FindObjectsOfTypeAll<Button>();
                int renamedCount = 0;
                foreach (Button button in buttons)
                {
                    if (button == null) continue;
                    string currentName = button.gameObject.name;
                    int suffixIndex = currentName.IndexOf("_intercepted", StringComparison.Ordinal);
                    if (suffixIndex < 0) continue;

                    button.gameObject.name = currentName.Substring(0, suffixIndex);
                    renamedCount++;
                }

                if (interceptors.Length > 0 || renamedCount > 0)
                {
                    _log.LogInfo($"[RuntimeDriver] Removed {interceptors.Length} interceptor component(s) and restored {renamedCount} button name(s).");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] UiEventInterceptor cleanup failed: {ex}");
            }
        }

        /// <summary>
        /// Activates the IMGUI fallback UI (ModMenuOverlay + ModSettingsPanel + HudIndicator).
        /// Safe to call from Update() as well as Initialize().
        /// No-ops if already activated.
        /// </summary>
        private void ActivateImguiFallback()
        {
            // Guard: only activate once
            if (_modMenuHost != null) return;

            try
            {
                ModMenuOverlay overlay = gameObject.AddComponent<ModMenuOverlay>();
                ModSettingsPanel settingsPanel = gameObject.AddComponent<ModSettingsPanel>();

                // Wire settings panel into mod menu for its inline Settings button
                overlay.SetSettingsPanel(settingsPanel);

                if (_modPlatform != null)
                {
                    _modPlatform.SetUI(overlay, settingsPanel);
                }

                // Wire the active menu host into NativeMenuInjector for the native Mods button
                if (_nativeMenuInjector != null)
                {
                    _nativeMenuInjector.SetModMenuHost(overlay);
                }

                _modMenuHost = overlay;
                _modSettingsHost = settingsPanel;

                _log.LogInfo("[RuntimeDriver] IMGUI fallback — Added ModMenuOverlay + ModSettingsPanel.");
            }
            catch (Exception ex)
            {
                _log.LogError($"[RuntimeDriver] IMGUI fallback ModMenuOverlay setup failed: {ex}");
            }

            try
            {
                _hudIndicator = gameObject.AddComponent<HudIndicator>();
                _hudIndicator.SetModMenu(_modMenuHost);

                if (_modMenuHost != null)
                {
                    _modMenuHost.OnReloadRequested += () => _hudIndicator?.ShowToast("Packs reloaded");
                }

                // Wire HudIndicator so IMGUI counter also receives pack counts on every load/reload.
                if (_modPlatform != null)
                {
                    HudIndicator hud = _hudIndicator;
                    _modPlatform.OnHudCountsChanged = (p, e) => hud.UpdateCounts(p, e);
                }

                _log.LogInfo("[RuntimeDriver] IMGUI fallback — Added HudIndicator.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] HudIndicator setup failed: {ex}");
            }
        }

        /// <summary>
        /// Wires UGUI DFCanvas to ModPlatform once DFCanvas.Start() has succeeded.
        /// Called the first frame that DFCanvas.IsReady becomes true.
        /// </summary>
        private void WireUguiToModPlatform()
        {
            if (_dfCanvas == null || _modPlatform == null) return;

            try
            {
                if (_dfCanvas.ModMenuPanel != null)
                {
                    _dfCanvas.ModMenuPanel.OnReloadRequested = () => _modPlatform?.LoadPacks();
                }

                IModSettingsHost settingsHost = new NoOpSettingsHost();

                if (_dfCanvas.ModMenuPanel == null)
                {
                    throw new InvalidOperationException("DFCanvas did not create ModMenuPanel.");
                }

                _modPlatform.SetUI(_dfCanvas.ModMenuPanel, settingsHost);

                // Fix #30: route the native Mods button through ContextualModMenuHost so
                // that when NativeMainMenuModMenu.CanUseNativeScreen becomes true (M11.5),
                // the native menu takes over automatically without re-wiring.
                // For now CanUseNativeScreen returns false, so overlay is still used.
                if (_nativeMenuInjector != null)
                {
                    NativeMainMenuModMenu nativeHost = new NativeMainMenuModMenu();
                    ContextualModMenuHost contextualHost = new ContextualModMenuHost(
                        _dfCanvas.ModMenuPanel, nativeHost);
                    _nativeMenuInjector.SetModMenuHost(contextualHost);
                    _log.LogInfo("[RuntimeDriver] NativeMenuInjector wired via ContextualModMenuHost (native stub active, overlay fallback).");
                }

                // Wire UGUI DebugPanel to ModPlatform so it displays platform status
                if (_dfCanvas.DebugPanel != null && _modPlatform != null)
                {
                    _dfCanvas.DebugPanel.SetModPlatform(_modPlatform);
                    _log.LogInfo("[RuntimeDriver] UGUI DebugPanel wired to ModPlatform.");
                }

                _modMenuHost = _dfCanvas.ModMenuPanel;
                _modSettingsHost = settingsHost;

                // Wire HudStrip so it receives pack counts on every load/reload.
                if (_dfCanvas.HudStrip != null)
                {
                    UI.HudStrip hudStrip = _dfCanvas.HudStrip;
                    _modPlatform.OnHudCountsChanged = (p, e) => hudStrip.SetStatus(p, e);
                }

                _log.LogInfo("[RuntimeDriver] UGUI wired to ModPlatform via IModMenuHost.");

                // Fix #31/#32: LoadPacks() may have run before the UI host was wired
                // (ModPlatform.UpdateUI() returns early when _modMenuHost is null).
                // Now that the host is registered, replay a LoadPacks() so ModMenuPanel
                // receives the pack list and DebugPanel receives ModPlatform data.
                // This is a no-op if packs have not been loaded yet.
                if (_modPlatform.GetLoadedPackIds() != null)
                {
                    _log.LogInfo("[RuntimeDriver] Replaying LoadPacks() to populate UGUI panels after late wiring.");
                    _modPlatform.LoadPacks();
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] UGUI→ModPlatform wiring failed, activating IMGUI fallback: {ex}");
                _uguiReady = false;
                ActivateImguiFallback();
            }
        }

        /// <summary>
        /// Called once when the ECS World becomes available (non-InitialGameLoader scenes only).
        /// Loads packs, starts hot reload. KeyInputSystem is registered every poll cycle
        /// via <see cref="TryRegisterKeyInputSystem"/> so it survives scene transitions.
        /// </summary>
        private void OnWorldReady(World ecsWorld)
        {
            _log.LogInfo($"[RuntimeDriver] ECS World available: {ecsWorld.Name}");
            _registeredWorldInstance = ecsWorld;

            // Register DumpSystem if configured
            if (_dumpOnStartup)
            {
                try
                {
                    DumpSystem.Configure(_log, _dumpOutputPath);
                    ecsWorld.GetOrCreateSystem<DumpSystem>();
                    _log.LogInfo("[RuntimeDriver] DumpSystem registered in default world.");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] DumpSystem registration failed: {ex}");
                }
            }

            // Notify ModPlatform that the world is ready
            if (_modPlatform != null)
            {
                try
                {
                    _modPlatform.OnWorldReady(ecsWorld);
                    _log.LogInfo("[RuntimeDriver] ModPlatform notified of world readiness.");
                }
                catch (Exception ex)
                {
                    _log.LogError($"[RuntimeDriver] ModPlatform.OnWorldReady failed: {ex}");
                }

                // Load packs
                try
                {
                    ContentLoadResult result = _modPlatform.LoadPacks();
                    _log.LogInfo($"[RuntimeDriver] Pack loading complete: success={result.IsSuccess}, " +
                        $"loaded={result.LoadedPacks.Count}, errors={result.Errors.Count}");
                }
                catch (Exception ex)
                {
                    _log.LogError($"[RuntimeDriver] Pack loading failed: {ex}");
                }

                // Start hot reload
                try
                {
                    _modPlatform.StartHotReload();
                    _log.LogInfo("[RuntimeDriver] Hot reload started.");
                }
                catch (Exception ex)
                {
                    _log.LogError($"[RuntimeDriver] Hot reload startup failed: {ex}");
                }

                // Discover settings for the settings panel
                try
                {
                    if (_modSettingsHost is ModSettingsPanel settingsPanel)
                    {
                        settingsPanel.DiscoverSettings();
                        _log.LogInfo("[RuntimeDriver] Mod settings discovered.");
                    }
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] Settings discovery failed: {ex}");
                }
            }

            // Give the debug overlay a reference to ModPlatform for status display
            if (_debugOverlay != null)
            {
                _debugOverlay.SetModPlatform(_modPlatform);
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                System.IO.File.AppendAllText(debugLog, $"[{System.DateTime.UtcNow:o}] {msg}\n");
            }
            catch { } // safe-swallow: best-effort debug I/O, non-critical
        }

        private void OnDestroy()
        {
            // Iter-144 #543 gray-freeze fix: signal all subsystems IMMEDIATELY, before any other
            // teardown work runs. VanillaCatalog.Build + ContentLoader pack registration check
            // this static flag and short-circuit cleanly to avoid racing world teardown.
            s_isBeingDestroyed = true;
            _destroyed = true; // Signal background polling thread to stop
            _backgroundPollStopEvent.Set();  // Wake up the polling loop

            // Iter-144 #547 H5: set resurrection flags BEFORE any work that could wedge. If
            // ShutdownNonBridge or any subsequent teardown step deadlocks the main thread, the
            // resurrection flag has already been observed by the Win32 fallback thread + the
            // activeSceneChanged subscriber (kept alive across BepInEx Plugin instance death).
            Plugin.NeedsResurrection = true;
            Plugin.NeedsDeferredResurrection = true;
            Plugin.PersistentRoot = null;

            // Honest reporting (iter-144 #535 re-fix): the previous "Bridge kept alive" claim was
            // misleading. What actually happens at this point:
            //   - The background polling thread (which runs OnWorldReady, catalog rebuild, world
            //     change detection) STOPS — this RuntimeDriver instance is dead.
            //   - The MainThread pump anchored on this driver's KeyInputSystem also stops servicing
            //     dispatches until TryResurrect attaches a new driver + KeyInputSystem to the new
            //     ECS world.
            //   - The GameBridgeServer thread (IsBackground=false, owned by Plugin.SharedBridgeServer)
            //     DOES survive. Verified at log time below. New requests sit in the pipe queue and
            //     will be serviced once TryResurrect installs a new pump.
            // Resurrection is initiated by SceneManager.activeSceneChanged (iter-144 #546 fix) +
            // a Win32 fallback thread (Plugin.ResurrectionFallbackLoop) + the background-poll deferred path.
            bool bridgeThreadAlive = false;
            try
            {
                Bridge.GameBridgeServer? srv = Plugin.SharedBridgeServer;
                bridgeThreadAlive = srv != null && srv.IsServerThreadAlive;
            }
            catch { } // safe-swallow: diagnostic-only liveness probe must not throw from OnDestroy
            WriteDebug(
                "[RuntimeDriver] OnDestroy: background poll stopped, main-thread pump idle until resurrection. " +
                $"BridgeServerThreadAlive={bridgeThreadAlive}. NeedsResurrection set; awaiting scene transition.");
            // IMPORTANT: Do NOT call _modPlatform.Shutdown() here.
            // The bridge server runs on its own thread and must survive RuntimeDriver destruction.
            // It will be reattached when TryResurrect creates a new RuntimeDriver.
            // Iter-144 #547 H5: dispatch ShutdownNonBridge to a worker thread so this OnDestroy
            // returns immediately. The dispose work (file watcher + HMR cleanup) is non-essential
            // for resurrection and was previously the suspect for native deadlock. Running it on
            // a background thread releases Unity's destruction pump even if dispose work wedges.
            try
            {
                ModPlatform? mp = _modPlatform;
                if (mp != null)
                {
                    WriteDebug("[RuntimeDriver] OnDestroy: dispatching ShutdownNonBridge to worker thread.");
                    Thread shutdownWorker = new Thread(() =>
                    {
                        try
                        {
                            mp.ShutdownNonBridge();
                            WriteDebug("[RuntimeDriver] OnDestroy.worker: ShutdownNonBridge completed.");
                        }
                        catch (Exception ex)
                        {
                            WriteDebug($"[RuntimeDriver] OnDestroy.worker: ShutdownNonBridge threw {ex.GetType().Name}: {ex.Message}");
                        }
                    })
                    {
                        Name = "DINOForge.ShutdownNonBridge",
                        IsBackground = true,
                    };
                    shutdownWorker.Start();
                }
            }
            catch (Exception ex)
            {
                WriteDebug($"[RuntimeDriver] OnDestroy: ShutdownNonBridge dispatch failed: {ex.Message}");
            }
            WriteDebug("[RuntimeDriver] OnDestroy: returning to Unity (resurrection flags set, fallback thread will revive).");
        }
    }
}
