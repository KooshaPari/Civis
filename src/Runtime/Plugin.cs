#nullable enable
using System;
using System.IO;
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

            // Harmony (available but unused per ADR-005)
            try
            {
                _harmony = new Harmony(PluginInfo.GUID);
                Log.LogInfo("Harmony initialized (no patches applied).");
            }
            catch (Exception ex)
            {
                Log.LogError($"Harmony init failed: {ex.Message}");
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

            // Capture state for static resurrection callback
            _resurrectionLog = Logger;
            _resurrectionConfig = Config;
            _resurrectionDump = dumpOnStartup.Value;
            _resurrectionDumpPath = dumpOutputPath.Value;

            // Register static SceneManager callbacks — survives all scene transitions and
            // GameObject destruction. Recreates DINOForge_Root if DINO's loader destroys it.
            SceneManager.sceneLoaded -= OnSceneLoaded;
            SceneManager.sceneLoaded += OnSceneLoaded;
            SceneManager.activeSceneChanged -= OnActiveSceneChanged;
            SceneManager.activeSceneChanged += OnActiveSceneChanged;
            SceneManager.sceneUnloaded -= OnSceneUnloaded;
            SceneManager.sceneUnloaded += OnSceneUnloaded;

            Log.LogInfo("[Plugin] SceneManager resurrection hooks registered.");

            WriteDebug("Awake completed");
            Log.LogInfo("DINOForge Runtime loaded successfully.");
        }

        /// <summary>
        /// SceneManager.sceneLoaded callback — fires after every scene load.
        /// Recreates DINOForge_Root if DINO's InitialGameLoader destroyed it.
        /// This is a static event registration that survives all GameObject destruction.
        /// </summary>
        private static void OnSceneLoaded(Scene scene, LoadSceneMode mode)
        {
            WriteDebug($"[Plugin] OnSceneLoaded: '{scene.name}' — PersistentRoot={(PersistentRoot == null ? "null" : "alive")}");
            TryResurrect(scene.name, "sceneLoaded");
        }

        /// <summary>
        /// SceneManager.activeSceneChanged callback — fires after the active scene switches.
        /// This fires AFTER the destroy sweep that kills DINOForge_Root, so resurrection here
        /// creates a new root that survives into the new scene.
        /// </summary>
        private static void OnActiveSceneChanged(Scene prev, Scene next)
        {
            WriteDebug($"[Plugin] OnActiveSceneChanged: '{prev.name}' → '{next.name}' — PersistentRoot={(PersistentRoot == null ? "null" : "alive")}");
            TryResurrect(next.name, "activeSceneChanged");
        }

        private static void OnSceneUnloaded(Scene scene)
        {
            WriteDebug($"[Plugin] OnSceneUnloaded: '{scene.name}' — PersistentRoot={(PersistentRoot == null ? "null" : "alive")}");
            TryResurrect(scene.name, "sceneUnloaded");
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
        private DFCanvas? _dfCanvas;

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
        private bool _uguiReady;
        // _uguiChecked: we only need to check DFCanvas readiness once after it has
        // had at least one frame to run its Start().
        private bool _uguiChecked;

        private bool _worldFound;
        private bool _initialized;
        private bool _catalogRebuilt;
        private float _worldPollTimer;

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

            // ── Wire KeyInputSystem ECS callbacks ───────────────────────────────────
            // KeyInputSystem lives in the ECS world and survives scene transitions.
            // It calls these lambdas when F9/F10 are pressed, even after DINOForge_Root
            // has been resurrected (the lambdas capture `this` — the current driver).
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

            // ── Step 2: Attempt UGUI canvas setup ───────────────────────────────────
            // DFCanvas.Initialize() only stores the logger; the actual canvas hierarchy
            // is built in DFCanvas.Start() which Unity calls on the NEXT frame.
            // We register an OnInitFailed callback so that if Start() throws, we can
            // activate the IMGUI fallback from Update() on the same frame it fails.
            bool uguiAddedOk = false;
            try
            {
                _dfCanvas = gameObject.AddComponent<DFCanvas>();
                _dfCanvas.Initialize(_log);

                // Register failure callback — called by DFCanvas.Start() if BuildCanvas throws.
                _dfCanvas.OnInitFailed = () =>
                {
                    _log.LogWarning("[RuntimeDriver] DFCanvas.OnInitFailed — activating IMGUI fallback.");
                    _uguiReady = false;
                    _uguiChecked = true;
                    ActivateImguiFallback();
                };

                uguiAddedOk = true;
                _log.LogInfo("[RuntimeDriver] Added DFCanvas — canvas hierarchy deferred to Start().");
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

            // ── Step 4: Log key handler registration ────────────────────────────────
            _log.LogInfo($"[RuntimeDriver] F9/F10 key handlers registered on {gameObject.name}.");
            _log.LogInfo("[RuntimeDriver] Waiting for ECS World (Update polling)...");
        }

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

                // Wire the active UGUI menu host into NativeMenuInjector for the native Mods button
                if (_nativeMenuInjector != null)
                {
                    _nativeMenuInjector.SetModMenuHost(_dfCanvas.ModMenuPanel);
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
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] UGUI→ModPlatform wiring failed, activating IMGUI fallback: {ex.Message}");
                _uguiReady = false;
                ActivateImguiFallback();
            }
        }

        private void Update()
        {
            // Wrap entire Update in try/catch — a logger exception (e.g. BepInEx log writer
            // failure) must NOT propagate out of Update() or Unity will silently disable this
            // MonoBehaviour forever, killing F9/F10 and all ECS polling.
            try
            {
            if (!_initialized) return;

            // Debug: log every ~5 seconds to confirm Update is running
            if (Time.frameCount % 300 == 0)
            {
                try { _log.LogInfo($"[RuntimeDriver] Update running frame {Time.frameCount}"); } catch { }
                WriteDebug($"[RuntimeDriver] Update heartbeat frame {Time.frameCount}");
            }

            // F9/F10 — direct input polling, always works as long as this MonoBehaviour is alive
            if (Input.GetKeyDown(KeyCode.F9))
            {
                try
                {
                    WriteDebug("[RuntimeDriver] F9 pressed");
                    if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleDebug();
                    else _debugOverlay?.Toggle();
                }
                catch { }
            }

            if (Input.GetKeyDown(KeyCode.F10))
            {
                try
                {
                    WriteDebug("[RuntimeDriver] F10 pressed");
                    if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
                    else _modMenuHost?.Toggle();
                }
                catch { }
            }

            // ── Check whether DFCanvas.Start() has run yet ───────────────────────
            // DFCanvas.Start() is deferred by Unity so we can't know in Initialize()
            // whether UGUI succeeded.  We check IsReady each frame until confirmed.
            if (!_uguiChecked && _dfCanvas != null)
            {
                if (_dfCanvas.IsReady)
                {
                    _uguiReady = true;
                    _uguiChecked = true;
                    _log.LogInfo("[RuntimeDriver] DFCanvas confirmed ready — UGUI active.");
                    WireUguiToModPlatform();
                }
                // If not yet ready, keep waiting (OnInitFailed callback handles failure)
            }

            // ── Deferred VanillaCatalog rebuild once the game scene is loaded ────
            // The world exists at startup with only 24 entities (loading screen).
            // We wait until entities > 1000 (full scene loaded ~45K entities) to rebuild.
            if (_worldFound && !_catalogRebuilt)
            {
                try
                {
                    World? w = World.DefaultGameObjectInjectionWorld;
                    if (w != null && w.IsCreated)
                    {
                        int entityCount = w.EntityManager.UniversalQuery.CalculateEntityCount();
                        if (entityCount > 1000)
                        {
                            _catalogRebuilt = true;
                            _modPlatform?.RebuildCatalogAndApplyStats(w);
                        }
                    }
                }
                catch { }
            }

            // ── ECS world poll ───────────────────────────────────────────────────
            if (_worldFound) return;

            _worldPollTimer += Time.deltaTime;
            if (_worldPollTimer < WorldPollInterval) return;
            _worldPollTimer = 0f;

            try
            {
                World? world = World.DefaultGameObjectInjectionWorld;
                if (world != null && world.IsCreated)
                {
                    _worldFound = true;
                    OnWorldReady(world);
                }
            }
            catch
            {
                // World not ready yet, will retry next poll
            }

            } // end outer try
            catch (Exception updateEx)
            {
                WriteDebug($"[RuntimeDriver] Update() exception (component kept alive): {updateEx.Message}");
            }
        }

        /// <summary>
        /// Called once when the ECS World becomes available.
        /// Registers systems, loads packs, starts hot reload.
        /// </summary>
        private void OnWorldReady(World ecsWorld)
        {
            _log.LogInfo($"[RuntimeDriver] ECS World available: {ecsWorld.Name}");

            // Register KeyInputSystem — handles F9/F10 via ECS (survives scene transitions)
            try
            {
                ecsWorld.GetOrCreateSystem<Bridge.KeyInputSystem>();
                _log.LogInfo("[RuntimeDriver] KeyInputSystem registered in default world.");
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] KeyInputSystem registration failed: {ex.Message}");
            }

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
            // DINO destroys all GameObjects during InitialGameLoader even with HideAndDontSave.
            // Immediate resurrection: create a new root and RuntimeDriver right here in OnDestroy.
            // This is safe — Unity allows new GameObjects to be created during OnDestroy callbacks.
            WriteDebug("[RuntimeDriver] OnDestroy — recreating DINOForge_Root immediately.");
            try
            {
                Plugin.PersistentRoot = null;
                GameObject root = new GameObject("DINOForge_Root");
                root.hideFlags = HideFlags.HideAndDontSave;
                UnityEngine.Object.DontDestroyOnLoad(root);
                Plugin.PersistentRoot = root;

                RuntimeDriver driver = root.AddComponent<RuntimeDriver>();
                driver.Initialize(_log, _config, _dumpOnStartup, _dumpOutputPath);
                WriteDebug("[RuntimeDriver] OnDestroy resurrection complete — new root created.");
            }
            catch (Exception ex)
            {
                WriteDebug($"[RuntimeDriver] OnDestroy resurrection FAILED: {ex.Message}");
            }
        }
    }
}
