#nullable enable
using System;
using System.IO;
using BepInEx;
using BepInEx.Configuration;
using BepInEx.Logging;
using DINOForge.Runtime.UI;
using DINOForge.SDK;
using DINOForge.SDK.Diagnostics;
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

        /// <summary>
        /// Static singleton bridge server that survives RuntimeDriver destruction.
        /// Created once, thread owned by Plugin class (not by any MonoBehaviour).
        /// </summary>
        internal static Bridge.GameBridgeServer? SharedBridgeServer;

        private void Awake()
        {
            Log = Logger;
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
                Log.LogWarning($"Version detection failed: {ex.Message}");
            }

            // Harmony — apply patches from this assembly
            // ModsButtonTextPatch (UI/UiGridHarmonyPatch.cs) intercepts Text/TMP_Text setters
            // to prevent DINO's UiGrid from overwriting our repurposed Mods button label.
            try
            {
                _harmony = new Harmony(PluginInfo.GUID);
                Bridge.DestroyGuardPatch.Apply(_harmony);
                UI.ModsButtonTextPatch.Apply(_harmony);
                Log.LogInfo("Harmony initialized and patches applied.");
            }
            catch (Exception ex)
            {
                Log.LogError($"Harmony init/patch failed: {ex.Message}");
            }

            // Sentry — initialize error tracking from SDK
            Diagnostics.SentryInitializer.Initialize(
                environment: Application.isPlaying ? "production" : "development",
                releaseOverride: PluginInfo.VERSION);
            SentryInitializer.AddBreadcrumb($"DINOForge Runtime v{PluginInfo.VERSION} initializing", "runtime");

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
                Log.LogError($"[Plugin] Failed to create persistent root: {ex.Message}");
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
                Log.LogError($"[Plugin] RuntimeDriver setup failed: {ex.Message}");
            }

            // Capture state for static resurrection callback (kept for emergency use)
            _resurrectionLog = Logger;
            _resurrectionConfig = Config;
            _resurrectionDump = dumpOnStartup.Value;
            _resurrectionDumpPath = dumpOutputPath.Value;

            StartResurrectionWatcher();

            WriteDebug("Awake completed");
            Log.LogInfo("DINOForge Runtime loaded successfully.");
        }

        /// <summary>
        /// Registers a static SceneManager.activeSceneChanged callback that re-registers
        /// KeyInputSystem in the current ECS world whenever a new scene finishes loading.
        /// This is the most reliable scene-transition detector — fires for every scene load
        /// including InitialGameLoader→gameplay where the ECS world changes.
        /// The callback persists as a static delegate even after Plugin MonoBehaviour destruction.
        /// </summary>
        private static void StartResurrectionWatcher()
        {
            SceneManager.sceneLoaded += OnSceneLoaded;
            WriteDebug("[Plugin] SceneLoaded watcher registered.");
        }

        private static void OnSceneLoaded(Scene scene, LoadSceneMode mode)
        {
            WriteDebug($"[Plugin] OnSceneLoaded: scene='{scene.name}' mode={mode}");
            Bridge.KeyInputSystem.RecreateInCurrentWorld();
            // RuntimeDriver may have been destroyed when DINO destroyed our root.
            // Trigger resurrection here so TryResurrect fires even if KeyInputSystem.OnCreate
            // doesn't fire (e.g. when RecreateInCurrentWorld finds no valid world).
            if (NeedsResurrection || PersistentRoot == null)
            {
                WriteDebug($"[Plugin] OnSceneLoaded: resurrection needed - NeedsRes={NeedsResurrection} rootNull={PersistentRoot == null}");
                TryResurrect(scene.name, "OnSceneLoaded");
            }
        }

        internal static void TryResurrect(string sceneName, string trigger)
        {
            if (PersistentRoot != null) return;

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
                File.AppendAllText(debugLog, $"[{DateTime.Now}] {msg}\n");
            }
            catch { }
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
            _harmony?.UnpatchSelf();
            WriteDebug("OnDestroy called (BepInEx object only)");
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
                _log.LogWarning($"[RuntimeDriver] TryRegisterKeyInputSystem failed: {ex.Message}");
            }
        }

        private bool _worldFound;
        private bool _initialized;
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
                _log.LogWarning($"[RuntimeDriver] UiAssets initialization failed: {ex.Message}");
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
                _log.LogError($"[RuntimeDriver] ModPlatform initialization failed: {ex.Message}");
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
                _log.LogError($"[RuntimeDriver] MainThreadDispatcher setup failed: {ex.Message}");
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
                _log.LogError($"[RuntimeDriver] DebugOverlayBehaviour setup failed: {ex.Message}");
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
                catch { }
            };
            Bridge.KeyInputSystem.OnF10Pressed = () =>
            {
                try
                {
                    WriteDebug("[RuntimeDriver] F10 pressed (via KeyInputSystem)");
                    if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
                    else _modMenuHost?.Toggle();
                }
                catch { }
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
                    _log?.LogWarning($"[RuntimeDriver] Pack reload failed: {ex.Message}");
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
                _log.LogWarning($"[RuntimeDriver] DFCanvas AddComponent failed, falling back to IMGUI immediately: {ex.Message}");

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
                _log.LogWarning($"[RuntimeDriver] NativeMenuInjector setup failed: {ex.Message}");
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
                            try { System.IO.File.Delete(signalPath); } catch { }

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
                                _log?.LogWarning($"[RuntimeDriver] HMR: Pack reload invocation failed: {ex.Message}");
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
                                _log?.LogWarning($"[RuntimeDriver] HMR: UGUI reset failed: {ex.Message}");
                            }

                            _log?.LogInfo("[RuntimeDriver] HMR: Reload complete.");
                        }
                    }
                }
                catch { }
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
                        System.Threading.Thread.Sleep(50); // Poll every 50ms

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
                                    _log?.LogWarning($"[RuntimeDriver] F10 toggle failed: {ex.Message}");
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
                            catch { }

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
                            catch { }
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
                                        _log?.LogWarning($"[RuntimeDriver] KeyInputSystem re-registration failed: {ex.Message}");
                                    }
                                }
                            }
                            catch { }
                        }
                    }
                }
                catch (System.Exception ex)
                {
                    _log?.LogError($"[RuntimeDriver] Background polling thread exception: {ex.Message}");
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
                _log.LogWarning($"[RuntimeDriver] UiEventInterceptor cleanup failed: {ex.Message}");
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
                _log.LogError($"[RuntimeDriver] IMGUI fallback ModMenuOverlay setup failed: {ex.Message}");
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
                _log.LogWarning($"[RuntimeDriver] HudIndicator setup failed: {ex.Message}");
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
                _log.LogWarning($"[RuntimeDriver] UGUI→ModPlatform wiring failed, activating IMGUI fallback: {ex.Message}");
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
                    _log.LogWarning($"[RuntimeDriver] DumpSystem registration failed: {ex.Message}");
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
                    _log.LogError($"[RuntimeDriver] ModPlatform.OnWorldReady failed: {ex.Message}");
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
                    _log.LogError($"[RuntimeDriver] Pack loading failed: {ex.Message}");
                }

                // Start hot reload
                try
                {
                    _modPlatform.StartHotReload();
                    _log.LogInfo("[RuntimeDriver] Hot reload started.");
                }
                catch (Exception ex)
                {
                    _log.LogError($"[RuntimeDriver] Hot reload startup failed: {ex.Message}");
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
                    _log.LogWarning($"[RuntimeDriver] Settings discovery failed: {ex.Message}");
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
                System.IO.File.AppendAllText(debugLog, $"[{System.DateTime.Now}] {msg}\n");
            }
            catch { }
        }

        private void OnDestroy()
        {
            _destroyed = true; // Signal background polling thread to stop
            WriteDebug("[RuntimeDriver] OnDestroy called — DINO destroyed our root. Bridge kept alive.");
            Plugin.NeedsResurrection = true;
            Plugin.PersistentRoot = null;
            // IMPORTANT: Do NOT call _modPlatform.Shutdown() here.
            // The bridge server runs on its own thread and must survive RuntimeDriver destruction.
            // It will be reattached when TryResurrect creates a new RuntimeDriver.
            // Only shut down file watchers and HMR (they depend on the MonoBehaviour lifecycle).
            try
            {
                _modPlatform?.ShutdownNonBridge();
            }
            catch { }
        }
    }
}
