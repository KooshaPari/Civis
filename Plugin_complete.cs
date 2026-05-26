#nullable enable
using System;
using System.Collections.Generic;
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
using UnityEngine.LowLevel;
using UnityEngine.UI;
using UnityEngine.SceneManagement;

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

        // Captured at Awake for resurrection — accessible to KeyInputSystem on main thread
        internal static ManualLogSource? ResurrectionLog;
        internal static ConfigFile? ResurrectionConfig;
        internal static bool ResurrectionDump;
        internal static string ResurrectionDumpPath = "";

        /// <summary>
        /// Tracks ECS worlds we've already registered KeyInputSystem in.
        /// Used by OnBeforeRenderWorldScan to avoid duplicate registrations.
        /// </summary>
        internal static readonly HashSet<string> _seenWorldKeys = new HashSet<string>();

        /// <summary>Flag set by KeyInputSystem when F9 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF9Toggle;

        /// <summary>Flag set by KeyInputSystem when F10 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF10Toggle;

        /// <summary>Flag indicating PersistentRoot needs resurrection.</summary>
        internal static volatile bool NeedsResurrection;

        /// <summary>Number of consecutive resurrection attempts since last successful resurrection.</summary>
        private static int _resurrectionAttempts;

        /// <summary>Maximum number of consecutive resurrection attempts before giving up.</summary>
        private const int MaxResurrectionAttempts = 3;

        /// <summary>Lock for non-blocking debug log file access (prevents deadlock with watcher thread).</summary>
        private static readonly object _debugLogLock = new object();

        /// <summary>Main thread ID captured in Awake. Used to guard Unity API calls from background threads.</summary>
        private static int _mainThreadId;

        /// <summary>Counter for PlayerLoop update heartbeat logging (every 600 calls).</summary>
        private static int _playerLoopUpdateCount;

        /// <summary>Counter for Camera.onPreCull heartbeat logging (every 600 calls).</summary>
        private static int _cameraPreCullCount;

        private void Awake()
        {
            Log = Logger;
            // Capture main thread ID for guarding Unity API calls from background threads
            _mainThreadId = System.Threading.Thread.CurrentThread.ManagedThreadId;
            WriteDebug($"[Plugin] Awake: mainThreadId={_mainThreadId}");
            Log.LogInfo($"DINOForge Runtime v{PluginInfo.VERSION} loading...");

            // Enable background rendering so the game continues to render frames
            // even when the window is not focused. Required for external screenshot capture.
            Application.runInBackground = true;

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
            ResurrectionLog = Logger;
            ResurrectionConfig = Config;
            ResurrectionDump = dumpOnStartup.Value;
            ResurrectionDumpPath = dumpOutputPath.Value;

            // Register onBeforeRender world scan immediately so new ECS worlds are picked up
            // even before RuntimeDriver is alive. Re-registered after each TryResurrect.
            // NOTE: Application.onBeforeRender may not fire in DINO's custom player loop,
            // but we register it as a belt-and-suspenders fallback.
            _seenWorldKeys.Clear();
            Application.onBeforeRender += OnBeforeRenderWorldScan;
            WriteDebug("[Plugin] Awake: OnBeforeRenderWorldScan registered");

            // Camera.onPreCull fires every frame when any camera renders — even in DINO's ECS loop.
            // This is our most reliable per-frame main-thread hook.
            Camera.onPreCull += OnCameraPreCull;
            WriteDebug("[Plugin] Camera.onPreCull registered");

            // Patch DINO's system groups as early as possible — these are the per-frame hooks
            PatchDinoSystemGroups();

            // Inject DINOForgePlayerLoopUpdate into Unity's PlayerLoop so resurrection
            // and world scanning can run every frame even when no MonoBehaviour is alive.
            // This is the primary resurrection mechanism (Application.onBeforeRender may not fire).
            InjectPlayerLoopUpdate();

            // NOTE: Do NOT register SceneManager callbacks. During scene teardown, these fire when Mono's
            // delegate system is corrupt. Even a no-op callback can trigger OTHER listeners (like
            // NativeMenuInjector) to execute unsafe code. Resurrection happens via KeyInputSystem
            // in the ECS loop, which is always main-thread-safe.
            WriteDebug("[Plugin] SceneManager callbacks NOT registered (unsafe during teardown)");

            StartResurrectionWatcher();

            WriteDebug("Awake completed");
            Log.LogInfo("DINOForge Runtime loaded successfully.");
        }


        /// <summary>
        /// Background thread for Win32 F9/F10 key polling and resurrection triggering.
        /// F9/F10 detection is done via GetAsyncKeyState() which is thread-safe.
        /// Also detects NeedsResurrection flag and triggers Application.onBeforeRender callback injection
        /// to ensure resurrection happens even if Application.onBeforeRender isn't firing naturally.
        /// </summary>
        private static void StartResurrectionWatcher()
        {
            WriteDebug("[Plugin] StartResurrectionWatcher: starting background thread.");
            Thread thread = new Thread(() =>
            {
                bool f9WasDown = false, f10WasDown = false;
                int loopCount = 0;

                WriteDebug("[Plugin] KeyInputWatcher thread started — Win32 polling + resurrection check");

                while (true)
                {
                    try { Thread.Sleep(50); }
                    catch (ThreadInterruptedException) { return; }
                    catch (ThreadAbortException) { Thread.ResetAbort(); }
                    catch { }
                    loopCount++;

                    // Heartbeat every 20 iterations (~1 second at 50ms)
                    if (loopCount % 20 == 0)
                    {
                        // Use ReferenceEquals (not Unity's == operator) — thread-safe from background thread
                        bool rootAlive = !System.Object.ReferenceEquals(PersistentRoot, null);
                        WriteDebug($"[Plugin] Watcher tick #{loopCount}: PersistentRoot={(rootAlive ? "ALIVE" : "NULL")}(ref={rootAlive}), NeedsResurrection={NeedsResurrection}");
                    }

                    // Win32 F9/F10 polling — NO Unity APIs
                    try
                    {
                        bool f9Down  = (GetAsyncKeyState(VK_F9)  & 0x8000) != 0;
                        bool f10Down = (GetAsyncKeyState(VK_F10) & 0x8000) != 0;

                        if (f9Down && !f9WasDown)
                        {
                            WriteDebug("[Plugin] Win32: F9 pressed — invoking OnF9Pressed directly");
                            PendingF9Toggle = true;
                            try { Bridge.KeyInputSystem.OnF9Pressed?.Invoke(); } catch { }
                        }
                        if (f10Down && !f10WasDown)
                        {
                            WriteDebug("[Plugin] Win32: F10 pressed — invoking OnF10Pressed directly");
                            PendingF10Toggle = true;
                            try { Bridge.KeyInputSystem.OnF10Pressed?.Invoke(); } catch { }
                        }

                        f9WasDown = f9Down;
                        f10WasDown = f10Down;
                    }
                    catch { }

                    // NeedsResurrection check
                    if (NeedsResurrection && System.Object.ReferenceEquals(PersistentRoot, null))
                    {
                        WriteDebug("[Plugin] Watcher: NeedsResurrection detected, scheduling TryResurrect");
                        NeedsResurrection = false;
                        try { TryResurrect("(Watcher)", "Watcher"); } catch { }
                    }

                    // Screenshot-on-demand: check every ~10 iterations (500ms).
                    // ScreenCapture.CaptureScreenshot() in Unity 2021.3 sets an internal flag
                    // that is processed at the next render frame — safe to call from any thread.
                    if (loopCount % 10 == 0)
                    {
                        try { CheckScreenshotRequest("[Watcher]"); } catch { }
                    }

                    // NOTE: NativeMenuInjector scanning is NOT done from this watcher thread —
                    // Resources.FindObjectsOfTypeAll<Canvas>() can deadlock when called from
                    // a background thread during Unity's asset-loading phase.
                    // Scans are triggered by OnActiveSceneChanged (main-thread-safe).
                }
            });
            thread.IsBackground = true;
            thread.Name = "DINOForge-KeyInputWatcher";
            thread.Start();
            WriteDebug("[Plugin] KeyInputWatcher thread launched — Win32 polling + resurrection check");
        }

        // Win32 GetAsyncKeyState for background thread F9/F10 polling
        private const int VK_F9 = 0x78;
        private const int VK_F10 = 0x79;

        [System.Runtime.InteropServices.DllImport("user32.dll")]
        private static extern short GetAsyncKeyState(int vKey);

        /// <summary>
        /// Non-blocking debug log writer using Monitor.TryEnter with 100ms timeout.
        /// If lock cannot be acquired in 100ms, silently skips this log entry to prevent deadlock.
        /// </summary>
        internal static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
                if (Monitor.TryEnter(_debugLogLock, 100))
                {
                    try
                    {
                        File.AppendAllText(debugLog, $"[{DateTime.Now:yyyy-MM-dd HH:mm:ss.fff}] {msg}\n");
                    }
                    finally
                    {
                        Monitor.Exit(_debugLogLock);
                    }
                }
                // If lock not acquired in 100ms, skip this log entry (avoid deadlock)
            }
            catch { }
        }


        /// <summary>
        /// Per-frame callback (via Application.onBeforeRender) that scans for new ECS worlds
        /// and registers KeyInputSystem in them. This callback is independent of MonoBehaviour
        /// lifecycle and fires even after RuntimeDriver resurrection.
        ///
        /// Replaces the background watcher thread's World.All iteration (which was deadlock-prone).
        /// </summary>
        private static int _onBeforeRenderCallCount;

        private static void OnSceneLoaded(UnityEngine.SceneManagement.Scene scene, UnityEngine.SceneManagement.LoadSceneMode mode)
        {
            try
            {
                WriteDebug($"[Plugin] OnSceneLoaded: '{scene.name}' (resuming from callback)");
                // UNSAFE: This callback is called during scene teardown when Mono's delegate system is corrupt.
                // We must NOT do ANY Unity API work here that triggers other callbacks or Mono delegates.
                // Even WriteDebug is risky, but it's the only safe operation left.
                //
                // DO NOT uncomment the code below — it will cause mono_runtime_delegate_invoke crashes:
                // TryResurrect(scene.name, "sceneLoaded");
                // ScanAndRegisterKeyInputSystem($"sceneLoaded:{scene.name}");
            }
            catch { }
        }


        private static void ScanAndRegisterKeyInputSystem(string context)
        {
            try
            {
                foreach (var w in World.All)
                {
                    if (w == null || !w.IsCreated) continue;
                    string key = $"{w.Name}#{w.GetHashCode()}";
                    if (_seenWorldKeys.Add(key))
                    {
                        WriteDebug($"[Plugin] {context}: new world '{w.Name}' — registering KeyInputSystem");
                        try
                        {
                            var kis = w.GetOrCreateSystem<Bridge.KeyInputSystem>();
                            try
                            {
                                var initGroup = w.GetOrCreateSystem<InitializationSystemGroup>();
                                initGroup.AddSystemToUpdateList(kis);
                                initGroup.SortSystems();
                                WriteDebug($"[Plugin] {context}: KeyInputSystem added to InitializationSystemGroup in '{w.Name}'");
                            }
                            catch (Exception groupEx)
                            {
                                WriteDebug($"[Plugin] {context}: group enrollment failed in '{w.Name}': {groupEx.Message}");
                            }
                        }
                        catch (Exception ex)
                        {
                            WriteDebug($"[Plugin] {context}: KeyInputSystem registration failed in '{w.Name}': {ex.Message}");
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                WriteDebug($"[Plugin] {context}: World.All scan threw: {ex.GetType().Name}: {ex.Message}");
            }
        }

        internal static void OnBeforeRenderWorldScan()
        {
            try
            {
                _onBeforeRenderCallCount++;
                // Log every 600 calls (~10s at 60fps) to confirm the callback is firing
                if (_onBeforeRenderCallCount <= 3 || _onBeforeRenderCallCount % 600 == 0)
                    WriteDebug($"[Plugin] OnBeforeRender #{_onBeforeRenderCallCount}: NeedsResurrection={NeedsResurrection}, PersistentRoot={(PersistentRoot != null ? "ALIVE" : "NULL")}");

                try
                {
                    // Check NeedsResurrection first — set by RuntimeDriver.OnDestroy after DINO's sweep.
                    // OnBeforeRender fires every frame on the main thread, so this is safe to call Unity APIs.
                    if (NeedsResurrection && PersistentRoot == null)
                    {
                        NeedsResurrection = false;
                        WriteDebug("[Plugin] OnBeforeRender: NeedsResurrection detected — calling TryResurrect");
                        TryResurrect("(OnBeforeRender)", "OnBeforeRender");
                    }
                }
                catch { }

                // Scan for new ECS worlds and register KeyInputSystem in them.
                // Guard: only iterate World.All if Unity is in a stable state (not during scene teardown).
                try
                {
                    foreach (var w in World.All)
                    {
                        if (w == null || !w.IsCreated) continue;
                        string key = $"{w.Name}#{w.GetHashCode()}";
                        if (_seenWorldKeys.Add(key))
                        {
                            WriteDebug($"[Plugin] OnBeforeRender: new world '{w.Name}' detected — registering KeyInputSystem");
                            try
                            {
                                var kis = w.GetOrCreateSystem<Bridge.KeyInputSystem>();
                                try
                                {
                                    var initGroup = w.GetOrCreateSystem<InitializationSystemGroup>();
                                    initGroup.AddSystemToUpdateList(kis);
                                    initGroup.SortSystems();
                                    WriteDebug($"[Plugin] KeyInputSystem added to InitializationSystemGroup in world '{w.Name}'");
                                }
                                catch (Exception groupEx)
                                {
                                    WriteDebug($"[Plugin] KeyInputSystem group enrollment failed in '{w.Name}': {groupEx.Message}");
                                }
                            }
                            catch (Exception ex)
                            {
                                WriteDebug($"[Plugin] OnBeforeRender: KeyInputSystem registration failed in '{w.Name}': {ex.Message}");
                            }
                        }
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug($"[Plugin] OnBeforeRender: World.All iteration threw: {ex.GetType().Name}: {ex.Message}");
                }
            }
            catch { }
        }

        private static void OnCameraPreCull(Camera cam)
        {
            try
            {
                _cameraPreCullCount++;

                // Heartbeat: first call + every 600 calls
                if (_cameraPreCullCount == 1 || _cameraPreCullCount % 600 == 0)
                {
                    WriteDebug($"[CameraPreCull] Tick #{_cameraPreCullCount}: cam={cam?.name}, PersistentRoot={(System.Object.ReferenceEquals(PersistentRoot, null) ? "NULL" : "ALIVE")}");
                }

                // F9/F10 via Input.GetKeyDown — main thread, safe to call Unity APIs
                if (Input.GetKeyDown(KeyCode.F9))
                {
                    WriteDebug("[CameraPreCull] F9 pressed");
                    Bridge.KeyInputSystem.OnF9Pressed?.Invoke();
                }
                if (Input.GetKeyDown(KeyCode.F10))
                {
                    WriteDebug("[CameraPreCull] F10 pressed");
                    Bridge.KeyInputSystem.OnF10Pressed?.Invoke();
                }

                // Resurrection check
                if (NeedsResurrection && System.Object.ReferenceEquals(PersistentRoot, null))
                {
                    WriteDebug("[CameraPreCull] NeedsResurrection detected, triggering TryResurrect");
                    TryResurrect("(CameraPreCull)", "CameraPreCull");
                }

                // Screenshot-on-demand: check trigger file every ~10 frames
                if (_cameraPreCullCount % 10 == 0)
                    CheckScreenshotRequest("[CameraPreCull]");
            }
            catch { }
        }

        internal static void TryResurrect(string sceneName, string trigger)
        {
            if (PersistentRoot != null) { NeedsResurrection = false; _resurrectionAttempts = 0; return; }

            // Guard: resurrection requires Unity APIs — must only run on the main thread
            if (_mainThreadId != 0 && System.Threading.Thread.CurrentThread.ManagedThreadId != _mainThreadId)
            {
                WriteDebug($"[Plugin] TryResurrect SKIPPED (background thread tid={System.Threading.Thread.CurrentThread.ManagedThreadId}). NeedsResurrection=true.");
                NeedsResurrection = true;
                return;
            }

            // Guard: stop looping forever if resurrection keeps failing
            if (_resurrectionAttempts >= MaxResurrectionAttempts)
            {
                if (_resurrectionAttempts == MaxResurrectionAttempts)
                {
                    WriteDebug($"[Plugin] TryResurrect: giving up after {MaxResurrectionAttempts} consecutive failures — resurrection loop halted.");
                    _resurrectionAttempts++; // increment past threshold to suppress log spam
                }
                return;
            }
            _resurrectionAttempts++;
            WriteDebug($"[Plugin] TryResurrect attempt {_resurrectionAttempts}/{MaxResurrectionAttempts} via {trigger} on '{sceneName}' — searching for host...");
            try
            {
                // Always create standalone GameObject — do not attach to camera
                GameObject host = new GameObject("DINOForge_Root");
                host.hideFlags = HideFlags.HideAndDontSave;
                UnityEngine.Object.DontDestroyOnLoad(host);
                PersistentRoot = host;
                WriteDebug($"[Plugin] TryResurrect: created new HideAndDontSave+DontDestroyOnLoad root");

                RuntimeDriver driver = host.AddComponent<RuntimeDriver>();
                driver.Initialize(ResurrectionLog!, ResurrectionConfig!, ResurrectionDump, ResurrectionDumpPath);

                // Log whether the new root is alive immediately after creation (from main thread, Unity == is safe)
                bool hostAliveUnity = (host != null); // Unity's == check (main-thread safe)
                bool hostAliveRef = !System.Object.ReferenceEquals(host, null); // C# reference check
                WriteDebug($"[Plugin] Post-resurrection host alive: Unity=={hostAliveUnity}, ReferenceEquals={hostAliveRef}, PersistentRoot=={(!System.Object.ReferenceEquals(PersistentRoot, null) ? "set" : "null")}");

                // Clear seen worlds so KeyInputSystem can be re-registered in the new ECS world
                _seenWorldKeys.Clear();
                WriteDebug($"[Plugin] TryResurrect: cleared seen worlds.");

                // Re-register onBeforeRender world scan for this resurrection context
                Application.onBeforeRender -= OnBeforeRenderWorldScan;
                Application.onBeforeRender += OnBeforeRenderWorldScan;
                WriteDebug("[Plugin] TryResurrect: registered OnBeforeRenderWorldScan");

                // Re-register Camera.onPreCull
                Camera.onPreCull -= OnCameraPreCull;
                Camera.onPreCull += OnCameraPreCull;
                WriteDebug("[Plugin] TryResurrect: registered Camera.onPreCull");

                // Reset attempt counter on success so future destruction cycles can resurrect again
                _resurrectionAttempts = 0;
                NeedsResurrection = false;
                WriteDebug($"[Plugin] Resurrection complete via {trigger} on '{sceneName}' host='{host.name}'.");
            }
            catch (Exception ex)
            {
                WriteDebug($"[Plugin] Resurrection FAILED via {trigger}: {ex.Message}");
            }
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

        /// <summary>
        /// Injects a custom player loop system into Unity's main update loop.
        /// This runs every frame at the engine level, independent of MonoBehaviour.Update() calls,
        /// and survives DINO's MonoBehaviour destruction.
        ///
        /// The injected system:
        /// - Checks for F9/F10 key presses and invokes KeyInputSystem callbacks
        /// - Checks if PersistentRoot needs resurrection and triggers TryResurrect
        /// - Logs a heartbeat every 600 calls for observability
        /// </summary>
        private static void InjectPlayerLoopUpdate()
        {
            try
            {
                var playerLoop = PlayerLoop.GetCurrentPlayerLoop();
                var newSubsystems = new List<PlayerLoopSystem>(playerLoop.subSystemList ?? System.Array.Empty<PlayerLoopSystem>());

                // Find the "Update" subsystem in the root loop
                var updateSystemIndex = -1;
                for (int i = 0; i < newSubsystems.Count; i++)
                {
                    if (newSubsystems[i].type == typeof(UnityEngine.PlayerLoop.Update))
                    {
                        updateSystemIndex = i;
                        break;
                    }
                }

                if (updateSystemIndex >= 0)
                {
                    var updateSystem = newSubsystems[updateSystemIndex];
                    var updateSubsystems = new List<PlayerLoopSystem>(updateSystem.subSystemList ?? System.Array.Empty<PlayerLoopSystem>());

                    // Create our custom player loop system
                    var dinoForgeSystem = new PlayerLoopSystem
                    {
                        type = typeof(DINOForgeUpdate),
                        updateDelegate = DINOForgePlayerLoopUpdate
                    };

                    // Append to the Update subsystem's list
                    updateSubsystems.Add(dinoForgeSystem);
                    updateSystem.subSystemList = updateSubsystems.ToArray();
                    newSubsystems[updateSystemIndex] = updateSystem;

                    // Set the modified player loop
                    playerLoop.subSystemList = newSubsystems.ToArray();
                    PlayerLoop.SetPlayerLoop(playerLoop);
                    PatchPlayerLoopRejection();
                    PatchDinoSystemGroups();

                    WriteDebug("[PlayerLoop] Injected DINOForgeUpdate into Update subsystem");
                    Log?.LogInfo("[PlayerLoop] F9/F10 input handler injected into player loop");
                }
                else
                {
                    WriteDebug("[PlayerLoop] WARNING: Could not find Update subsystem in player loop");
                    Log?.LogWarning("[PlayerLoop] Could not find Update subsystem for injection");
                }
            }
            catch (Exception ex)
            {
                WriteDebug($"[PlayerLoop] EXCEPTION during injection: {ex.Message}\n{ex.StackTrace}");
                Log?.LogError($"[PlayerLoop] PlayerLoop injection failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Custom player loop callback that runs every frame at engine level.
        /// </summary>
        private static void DINOForgePlayerLoopUpdate()
        {
            try
            {
                _playerLoopUpdateCount++;

                // First-tick diagnostic
                if (_playerLoopUpdateCount == 1)
                    WriteDebug("[PlayerLoop] FIRST TICK — PlayerLoop is firing!");

                // Check for F9 key press
                if (Input.GetKeyDown(KeyCode.F9))
                {
                    WriteDebug("[PlayerLoop] F9 pressed");
                    Bridge.KeyInputSystem.OnF9Pressed?.Invoke();
                }

                // Check for F10 key press
                if (Input.GetKeyDown(KeyCode.F10))
                {
                    WriteDebug("[PlayerLoop] F10 pressed");
                    Bridge.KeyInputSystem.OnF10Pressed?.Invoke();
                }

                // Check for resurrection need — PlayerLoop is the primary resurrection mechanism
                // since Application.onBeforeRender does not fire in DINO's custom player loop.
                if (NeedsResurrection && System.Object.ReferenceEquals(PersistentRoot, null))
                {
                    WriteDebug("[PlayerLoop] NeedsResurrection detected, triggering TryResurrect");
                    TryResurrect("(PlayerLoop)", "PlayerLoop");
                }

                // Also run the world scan every frame via PlayerLoop (mirrors OnBeforeRenderWorldScan)
                OnBeforeRenderWorldScan();

                // Screenshot-on-demand: check trigger file every ~10 frames from PlayerLoop
                if (_playerLoopUpdateCount % 10 == 0)
                {
                    CheckScreenshotRequest("[PlayerLoop]");
                }

                // Heartbeat every 600 calls (~10 seconds at 60 FPS)
                if (_playerLoopUpdateCount % 600 == 0)
                {
                    bool rootAlive = !System.Object.ReferenceEquals(PersistentRoot, null);
                    WriteDebug($"[PlayerLoop] Heartbeat #{_playerLoopUpdateCount}: PersistentRoot={(rootAlive ? "ALIVE" : "NULL")}");
                }
            }
            catch (Exception ex)
            {
                try
                {
                    WriteDebug($"[PlayerLoop] EXCEPTION in callback: {ex.Message}\n{ex.StackTrace}");
                }
                catch { }
            }
        }

        private static string? _screenshotPendingPath = null;

        private static void CheckScreenshotRequest(string context)
        {
            try
            {
                string bepRoot  = BepInEx.Paths.BepInExRootPath;
                string reqFile  = System.IO.Path.Combine(bepRoot, "dinoforge_screenshot_request.txt");
                string doneFile = System.IO.Path.Combine(bepRoot, "dinoforge_screenshot_done.txt");

                if (_screenshotPendingPath != null)
                {
                    // Previous ScreenCapture.CaptureScreenshot was called — write done file
                    System.IO.File.WriteAllText(doneFile, _screenshotPendingPath);
                    WriteDebug($"{context} Screenshot done: {_screenshotPendingPath}");
                    _screenshotPendingPath = null;
                }
                else if (System.IO.File.Exists(reqFile))
                {
                    string path = System.IO.File.ReadAllText(reqFile).Trim();
                    System.IO.File.Delete(reqFile);
                    if (string.IsNullOrEmpty(path))
                        path = System.IO.Path.Combine(bepRoot, "screenshot.png");
                    WriteDebug($"{context} Screenshot requested: {path}");
                    ScreenCapture.CaptureScreenshot(path);
                    _screenshotPendingPath = path;
                }
            }
            catch { }
        }

        private static bool _playerLoopHarmonyPatched = false;
        private static bool _dinoSystemGroupHarmonyPatched = false;
        private static int _dinoSystemGroupCallCount = 0;
        private static bool _inDinoSystemGroupUpdate = false;
        private static string? _pendingAutoScreenshot = null;
        private static int _pendingAutoScreenshotDelay = 0;

        private static void PatchPlayerLoopRejection()
        {
            if (_playerLoopHarmonyPatched) return;
            try
            {
                var harmony = new Harmony("dinoforge.plugin.playerloop");
                var original = typeof(PlayerLoop).GetMethod(
                    nameof(PlayerLoop.SetPlayerLoop),
                    System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (original == null)
                {
                    WriteDebug("[PlayerLoop] PatchPlayerLoopRejection: SetPlayerLoop method not found");
                    return;
                }
                var postfix = typeof(Plugin).GetMethod(
                    nameof(OnPlayerLoopSet),
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                harmony.Patch(original, postfix: new HarmonyMethod(postfix));
                _playerLoopHarmonyPatched = true;
                WriteDebug("[PlayerLoop] Harmony postfix on PlayerLoop.SetPlayerLoop applied (re-injection guard)");
            }
            catch (Exception ex)
            {
                WriteDebug($"[PlayerLoop] PatchPlayerLoopRejection failed: {ex.GetType().Name}: {ex.Message}");
            }
        }

        // Called after every PlayerLoop.SetPlayerLoop — re-injects our DINOForgeUpdate entry
        private static bool _reinjecting = false;
        private static void OnPlayerLoopSet()
        {
            if (_reinjecting) return;
            _reinjecting = true;
            try
            {
                InjectPlayerLoopUpdate();
            }
            finally
            {
                _reinjecting = false;
            }
        }

        /// <summary>
        /// Harmony prefix on DINO's concrete system group types from DNO.Main.dll.
        /// These groups are managed C# code (not Burst) and are called every frame
        /// by DINO's ECS scheduler — giving us guaranteed per-frame main-thread execution.
        /// We target multiple groups as fallbacks; the first one to fire wins.
        /// </summary>
        private static void PatchDinoSystemGroups()
        {
            if (_dinoSystemGroupHarmonyPatched) return;
            try
            {
                // Find DNO.Main assembly (already loaded by BepInEx at this point)
                System.Reflection.Assembly? dnoMain = null;
                foreach (var asm in AppDomain.CurrentDomain.GetAssemblies())
                {
                    if (asm.GetName().Name == "DNO.Main")
                    {
                        dnoMain = asm;
                        break;
                    }
                }

                if (dnoMain == null)
                {
                    WriteDebug("[DinoSysGroup] DNO.Main assembly not found in AppDomain");
                    return;
                }

                WriteDebug($"[DinoSysGroup] Found DNO.Main: {dnoMain.FullName}");

                var harmony = new Harmony("dinoforge.plugin.dinosysgroup");

                var postfix = typeof(Plugin).GetMethod(
                    nameof(OnDinoSystemGroupUpdate),
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);

                if (postfix == null)
                {
                    WriteDebug("[DinoSysGroup] OnDinoSystemGroupUpdate postfix method not found via reflection");
                    return;
                }

                // Target group type names in priority order (all in Systems.ComponentSystemGroups namespace)
                // FightGroup runs in SimulationSystemGroup every gameplay frame
                // GameplayInitializationSystemsGroup runs in InitializationSystemGroup — earliest in frame
                // PathFindingGroup runs every frame during gameplay
                string[] groupTypeNames = new[]
                {
                    "Systems.ComponentSystemGroups.FightGroup",
                    "Systems.ComponentSystemGroups.GameplayInitializationSystemsGroup",
                    "Systems.ComponentSystemGroups.PathFindingGroup",
                    "Systems.ComponentSystemGroups.ResourceDeliveryGroup",
                };

                int patchCount = 0;
                foreach (string typeName in groupTypeNames)
                {
                    try
                    {
                        var groupType = dnoMain.GetType(typeName);
                        if (groupType == null)
                        {
                            WriteDebug($"[DinoSysGroup] Type not found: {typeName}");
                            continue;
                        }

                        // OnUpdate() is the managed C# virtual method on ComponentSystemGroup
                        // that DINO's ECS scheduler calls every frame per group
                        var onUpdateMethod = groupType.GetMethod(
                            "OnUpdate",
                            System.Reflection.BindingFlags.Instance |
                            System.Reflection.BindingFlags.Public |
                            System.Reflection.BindingFlags.NonPublic);

                        if (onUpdateMethod == null)
                        {
                            // Try inherited from base class
                            onUpdateMethod = typeof(Unity.Entities.ComponentSystemGroup).GetMethod(
                                "OnUpdate",
                                System.Reflection.BindingFlags.Instance |
                                System.Reflection.BindingFlags.Public |
                                System.Reflection.BindingFlags.NonPublic);
                        }

                        if (onUpdateMethod == null)
                        {
                            WriteDebug($"[DinoSysGroup] OnUpdate() not found on {typeName}");
                            continue;
                        }

                        harmony.Patch(onUpdateMethod, postfix: new HarmonyMethod(postfix));
                        patchCount++;
                        WriteDebug($"[DinoSysGroup] Patched {typeName}.OnUpdate() successfully");

                        // Only need one successful patch — break after first
                        break;
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"[DinoSysGroup] Failed to patch {typeName}: {ex.GetType().Name}: {ex.Message}");
                    }
                }

                if (patchCount > 0)
                {
                    _dinoSystemGroupHarmonyPatched = true;
                    WriteDebug($"[DinoSysGroup] Harmony patch applied to {patchCount} DINO system group(s)");
                }
                else
                {
                    WriteDebug("[DinoSysGroup] No DINO system groups could be patched — all attempts failed");
                }
            }
            catch (Exception ex)
            {
                WriteDebug($"[DinoSysGroup] PatchDinoSystemGroups failed: {ex.GetType().Name}: {ex.Message}");
            }
        }

        // Postfix called after every DINO system group OnUpdate() — runs on main thread, every frame.
        private static void OnDinoSystemGroupUpdate()
        {
            if (_inDinoSystemGroupUpdate) return;
            _inDinoSystemGroupUpdate = true;
            try
            {
                _dinoSystemGroupCallCount++;

                // Heartbeat: first call + every 600 calls (~10s at 60fps)
                if (_dinoSystemGroupCallCount == 1 || _dinoSystemGroupCallCount % 600 == 0)
                {
                    WriteDebug($"[DinoSysGroup] Tick #{_dinoSystemGroupCallCount}: PersistentRoot={(System.Object.ReferenceEquals(PersistentRoot, null) ? "NULL" : "ALIVE")}");
                }

                // F9/F10 via Input.GetKeyDown — main thread, safe to call Unity APIs
                if (Input.GetKeyDown(KeyCode.F9))
                {
                    WriteDebug("[DinoSysGroup] F9 pressed");
                    Bridge.KeyInputSystem.OnF9Pressed?.Invoke();
                    // Schedule auto-checkpoint screenshot on next frame (overlay needs 1 frame to render)
                    _pendingAutoScreenshot = "cp2_f9_overlay";
                    _pendingAutoScreenshotDelay = 3;
                }
                if (Input.GetKeyDown(KeyCode.F10))
                {
                    WriteDebug("[DinoSysGroup] F10 pressed");
                    Bridge.KeyInputSystem.OnF10Pressed?.Invoke();
                    // Schedule auto-checkpoint screenshot on next frame
                    _pendingAutoScreenshot = "cp3_f10_menu";
                    _pendingAutoScreenshotDelay = 3;
                }

                // Auto-checkpoint screenshot: take delayed screenshot after F9/F10
                if (_pendingAutoScreenshot != null && _pendingAutoScreenshotDelay > 0)
                {
                    _pendingAutoScreenshotDelay--;
                    if (_pendingAutoScreenshotDelay == 0)
                    {
                        string path = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, _pendingAutoScreenshot + ".png");
                        WriteDebug($"[DinoSysGroup] Auto-checkpoint screenshot: {path}");
                        ScreenCapture.CaptureScreenshot(path);
                        _pendingAutoScreenshot = null;
                    }
                }

                // Resurrection check
                if (NeedsResurrection && System.Object.ReferenceEquals(PersistentRoot, null))
                {
                    WriteDebug("[DinoSysGroup] NeedsResurrection detected, triggering TryResurrect");
                    TryResurrect("(DinoSysGroup)", "DinoSysGroup");
                }

                // Screenshot-on-demand
                if (_dinoSystemGroupCallCount % 10 == 0)
                    CheckScreenshotRequest("[DinoSysGroup]");
            }
            catch { }
            finally
            {
                _inDinoSystemGroupUpdate = false;
            }
        }

        /// <summary>
        /// Marker type for the PlayerLoop system injection.
        /// </summary>
        private struct DINOForgeUpdate { }

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
        private int _updateCallCount;

        // World scan for new ECS worlds (runs on main thread, safe from background thread)
        private float _worldScanTimer;
        private readonly HashSet<string> _registeredWorlds = new();

        /// <summary>Polling interval in seconds for ECS world detection.</summary>
        private const float WorldPollInterval = 0.5f;

        /// <summary>
        /// Initializes the driver with config and logger references.
        /// Called immediately after AddComponent by Plugin.Awake().
        /// </summary>
        public void Initialize(ManualLogSource log, ConfigFile config, bool dumpOnStartup, string dumpOutputPath)
        {
            WriteDebug($"[RuntimeDriver.Initialize] Entry: gameObject.activeInHierarchy={gameObject.activeInHierarchy}, enabled={enabled}, scene={gameObject.scene.name}, timeScale={Time.timeScale}, frame={Time.frameCount}");

            _log = log;
            _config = config;
            _dumpOnStartup = dumpOnStartup;
            _dumpOutputPath = dumpOutputPath;
            _initialized = true;

            WriteDebug($"[RuntimeDriver.Initialize] Post-init: _initialized={_initialized}");

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
            // DFCanvas.Initialize() now builds the canvas immediately (not deferred to Start())
            // because DINO never calls MonoBehaviour.Start() or Update().
            // OnInitFailed is set BEFORE calling Initialize() to handle synchronous failures.
            bool uguiAddedOk = false;
            try
            {
                _dfCanvas = gameObject.AddComponent<DFCanvas>();

                // Set failure callback BEFORE Initialize() since canvas now builds synchronously
                _dfCanvas.OnInitFailed = () =>
                {
                    _log.LogWarning("[RuntimeDriver] DFCanvas.OnInitFailed — activating IMGUI fallback.");
                    _uguiReady = false;
                    _uguiChecked = true;
                    ActivateImguiFallback();
                };

                _dfCanvas.Initialize(_log);

                if (_dfCanvas.IsReady)
                {
                    _uguiReady = true;
                    _uguiChecked = true;
                    _log.LogInfo("[RuntimeDriver] DFCanvas built synchronously — UGUI ready.");
                    WireUguiToModPlatform();
                }
                else
                {
                    _log.LogWarning("[RuntimeDriver] DFCanvas.IsReady=false after Initialize — falling back to IMGUI.");
                    _uguiChecked = true;
                    ActivateImguiFallback();
                }

                uguiAddedOk = true;
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
                // Wire the background-thread scan trigger — watcher calls this every ~5s
                NativeMenuInjector.OnScanNeeded = () =>
                {
                    try { _nativeMenuInjector?.TryInjectMenuButton(); } catch { }
                };
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

        private void OnEnable()
        {
            WriteDebug($"[RuntimeDriver.OnEnable] called, frame={Time.frameCount}");
        }

        private void OnDisable()
        {
            WriteDebug($"[RuntimeDriver.OnDisable] called, frame={Time.frameCount}");
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
            // Log first 3 calls and every 300 frames to confirm Update is running
            if (_updateCallCount++ < 3 || _updateCallCount % 300 == 0)
                WriteDebug($"[RuntimeDriver.Update] #{_updateCallCount} frame={Time.frameCount} initialized={_initialized}");

            if (!_initialized) return;

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

            // ── World scan: register KeyInputSystem in any worlds we haven't seen yet ────
            // This runs every 1 second on the main thread (safe from background thread).
            // Replaces the background watcher thread's World.All iteration (which was deadlock-prone).
            _worldScanTimer += Time.deltaTime;
            if (_worldScanTimer >= 1f)
            {
                _worldScanTimer = 0f;
                try
                {
                    foreach (var w in World.All)
                    {
                        if (w == null || !w.IsCreated) continue;
                        string key = $"{w.Name}#{w.GetHashCode()}";
                        if (_registeredWorlds.Add(key))
                        {
                            WriteDebug($"[RuntimeDriver.Update] New world '{w.Name}' detected — registering KeyInputSystem");
                            try
                            {
                                var kis = w.GetOrCreateSystem<Bridge.KeyInputSystem>();
                                try
                                {
                                    var initGroup = w.GetOrCreateSystem<InitializationSystemGroup>();
                                    initGroup.AddSystemToUpdateList(kis);
                                    initGroup.SortSystems();
                                    WriteDebug($"[RuntimeDriver.Update] KeyInputSystem added to InitializationSystemGroup in world '{w.Name}'");
                                }
                                catch (Exception groupEx)
                                {
                                    WriteDebug($"[RuntimeDriver.Update] KeyInputSystem group enrollment failed in '{w.Name}': {groupEx.Message}");
                                }
                            }
                            catch (Exception ex)
                            {
                                WriteDebug($"[RuntimeDriver.Update] KeyInputSystem registration failed in '{w.Name}': {ex.Message}");
                            }
                        }
                    }
                }
                catch (Exception worldEx)
                {
                    WriteDebug($"[RuntimeDriver.Update] World.All iteration threw: {worldEx.GetType().Name}: {worldEx.Message}");
                }
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

        private void LateUpdate()
        {
            // Logging removed — was flooding debug log at 60fps
        }

        /// <summary>
        /// Called once when the ECS World becomes available.
        /// Registers systems, loads packs, starts hot reload.
        /// </summary>
        private void OnWorldReady(World ecsWorld)
        {
            _log.LogInfo($"[RuntimeDriver] ECS World available: {ecsWorld.Name}");

            // Register KeyInputSystem — handles F9/F10 via ECS (survives scene transitions)
            // Group placement is handled via [UpdateInGroup(typeof(InitializationSystemGroup))] attribute on the system
            try
            {
                var kis = ecsWorld.GetOrCreateSystem<Bridge.KeyInputSystem>();
                try
                {
                    var initGroup = ecsWorld.GetOrCreateSystem<InitializationSystemGroup>();
                    initGroup.AddSystemToUpdateList(kis);
                    initGroup.SortSystems();
                    _log.LogInfo("[RuntimeDriver] KeyInputSystem added to InitializationSystemGroup in default world.");
                }
                catch (Exception groupEx)
                {
                    _log.LogWarning($"[RuntimeDriver] KeyInputSystem group enrollment failed: {groupEx.Message}");
                }
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

        /// <summary>
        /// Delegates to Plugin.WriteDebug (non-blocking, prevents deadlock).
        /// </summary>
        private static void WriteDebug(string msg)
        {
            Plugin.WriteDebug(msg);
        }

        private void OnDestroy()
        {
            WriteDebug("[RuntimeDriver] OnDestroy called.");
            try { _modPlatform?.Shutdown(); }
            catch { }

            bool rootIsThis = System.Object.ReferenceEquals(Plugin.PersistentRoot, gameObject);
            WriteDebug($"[RuntimeDriver] OnDestroy: PersistentRoot=={(System.Object.ReferenceEquals(Plugin.PersistentRoot, null) ? "null" : "set")}, rootIsThis={rootIsThis}");

            if (!rootIsThis)
            {
                WriteDebug("[RuntimeDriver] OnDestroy: PersistentRoot is a different host — no resurrection needed.");
                return;
            }

            // Immediate resurrection inside OnDestroy — the ONLY reliable approach in DINO.
            // Unity allows new GameObject creation during OnDestroy; the new object is NOT
            // included in the current destroy sweep and will survive into the next frame.
            // All other approaches (PlayerLoop, SyncContext, watcher) never get execution time
            // because DINO suppresses Unity's Update loop during its ECS initialization.
            Plugin.PersistentRoot = null;
            Plugin._seenWorldKeys.Clear();
            WriteDebug("[RuntimeDriver] OnDestroy: PersistentRoot was this — resurrecting immediately.");
            try
            {
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
                WriteDebug($"[RuntimeDriver] OnDestroy resurrection FAILED: {ex.GetType().Name}: {ex.Message}");
                Plugin.NeedsResurrection = true; // fallback
            }
        }
    }
}


