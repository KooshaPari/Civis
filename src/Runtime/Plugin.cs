#nullable enable
// Iter-144 #543 gray-freeze patch — pre-existing DF analyzer warnings in this file are
// outside the scope of the patch and tracked separately (see Pattern Catalog #105/#106/#111/#231).
#pragma warning disable DF0105 // event-lifecycle asymmetry (pre-existing, tracked)
#pragma warning disable DF0106 // implicit File.ReadAllText encoding (pre-existing, tracked)
#pragma warning disable DF0111 // empty catch block (pre-existing safe-swallows, tracked)
#pragma warning disable DF1006 // disposable field (pre-existing BepInEx-owned, tracked)
using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using System.Threading;
using BepInEx;
using BepInEx.Configuration;
using BepInEx.Logging;
using DINOForge.Runtime.Diagnostics;
using DINOForge.Runtime.Localization;
using DINOForge.Runtime.Telemetry;
using DINOForge.Runtime.UI;
using DINOForge.Runtime.Updates;
using DINOForge.SDK;
using HarmonyLib;
using Unity.Entities;
using UnityEngine;
using UnityEngine.LowLevel;
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
    [BepInPlugin(PluginInfo.GUID, PluginInfo.NAME, PluginInfo.BEPINEX_VERSION)]
    public class Plugin : BaseUnityPlugin
    {
        private static ManualLogSource Log = null!;
        private Harmony? _harmony;

        internal static ConfigEntry<bool>? _showOverlayOnStart;
        internal static ConfigEntry<string>? _graphicsTier;
        internal static Graphics.GraphicsMode? _graphicsMode;
        internal static ConfigEntry<bool>? _enableHotReload;
        internal static ConfigEntry<int>? _hmrDebounceMs;

        // Static constructor fires BEFORE Awake — probe entry point
        static Plugin()
        {
            try
            {
                string debugLog = Path.Combine(Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] [STATIC] Plugin class referenced\n"); // unbounded-log-ok: static ctor fires once per AppDomain load; one-shot probe before DebugLog is initialized — Pattern #232
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

        /// <summary>
        /// iter-149e: true once Plugin.Awake captured the resurrection parameters (_resurrectionLog /
        /// _resurrectionConfig). Until then, a direct main-thread TryResurrect would NPE on those.
        /// KeyInputSystem.OnCreate (a DINO-driven main-thread callback that fires when the MainMenu
        /// ECS world is created — proven to fire post-teardown) checks this before reviving directly.
        /// </summary>
        internal static bool ResurrectionParamsReady => _resurrectionLog != null && _resurrectionConfig != null;

        /// <summary>
        /// iter-149e: main-thread revive entry usable by DINO-driven callbacks (KeyInputSystem.OnCreate,
        /// PlayerLoop.SetPlayerLoop postfix) that fire after our own bg threads/scene events have gone
        /// silent. Delegates to <see cref="MainThreadReviveIfNeeded"/>. Caller MUST be on the Unity
        /// main thread (these callbacks are). Never throws (Pattern #104/#111).
        /// </summary>
        internal static void ReviveFromMainThreadCallback(string trigger)
        {
            try { MainThreadReviveIfNeeded(LastSceneNameForResurrection ?? "world-create", trigger); }
            catch (Exception ex)
            {
                try { DebugLog.Write("Plugin", $"[Plugin] ReviveFromMainThreadCallback ({trigger}) threw: {ex.GetType().Name}: {ex.Message}"); }
                catch { /* diagnostic only */ }
            }
        }

        /// <summary>Flag set by KeyInputSystem when F9 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF9Toggle;

        /// <summary>Flag set by KeyInputSystem when F10 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF10Toggle;

        /// <summary>Flag indicating PersistentRoot needs resurrection.</summary>
        internal static volatile bool NeedsResurrection;

        // ── Engine-driven heartbeat (iter-149e, 2026-05-29) ───────────────────────────
        // WinDbg MDMP (docs/sessions/engine-ui-windbg-mdmp-20260529.md) proved the wedge is a
        // DORMANT-PLUGIN lifecycle bug, NOT a native deadlock: the engine main thread is in the
        // normal Unity idle wait while the plugin's worker threads are gone. The old wedge
        // classifier ("log mtime frozen + process alive + Responding") could not distinguish a
        // benign-engine/dormant-plugin from a true native wedge. This heartbeat is incremented and
        // flushed to BepInEx/dinoforge_heartbeat.txt from EVERY reliable main-thread tick
        // (scene events + PlayerLoop). If the heartbeat keeps advancing while the plugin LOG is
        // frozen, it is a dormant-plugin lifecycle bug (this class). If both are frozen, it is a
        // native wedge (iter-144 class). Never misclassify again.
        private static long _engineHeartbeat;
        private static readonly object _engineHeartbeatLock = new object();
        private const string EngineHeartbeatFileName = "dinoforge_heartbeat.txt";

        /// <summary>
        /// Increments the engine heartbeat and best-effort flushes it to
        /// <c>BepInEx/dinoforge_heartbeat.txt</c>. Safe to call from any reliable main-thread tick
        /// (scene events, PlayerLoop). Never throws (Pattern #104/#111).
        /// </summary>
        internal static void BumpEngineHeartbeat(string source)
        {
            try
            {
                long n;
                lock (_engineHeartbeatLock)
                {
                    n = ++_engineHeartbeat;
                }
                // Throttle disk writes to ~once per N bumps to avoid I/O churn; callers gate cadence.
                string root = BepInEx.Paths.BepInExRootPath;
                if (string.IsNullOrEmpty(root)) return;
                string path = Path.Combine(root, EngineHeartbeatFileName);
                string body = n.ToString() + " " + DateTime.UtcNow.ToString("o") + " " + (source ?? "") + "\n";
                File.WriteAllText(path, body, System.Text.Encoding.UTF8);
            }
            catch (Exception ex)
            {
                try { DebugLog.Write("Plugin", $"[Heartbeat] write failed (non-fatal): {ex.GetType().Name}: {ex.Message}"); }
                catch { /* diagnostic only */ }
            }
        }

        /// <summary>Number of consecutive resurrection attempts since last successful resurrection.</summary>
        private static int _resurrectionAttempts;

        /// <summary>Maximum consecutive resurrection attempts before giving up (SPEC-004 KIS-NF4).</summary>
        private const int MaxResurrectionAttempts = 3;

        /// <summary>
        /// Iter-144 #543 fix: Companion flag set by RuntimeDriver.OnDestroy BEFORE any teardown work.
        /// Resurrection check OR's this with NeedsResurrection to avoid the Unity fake-null trap
        /// (PersistentRoot field may hold a destroyed-but-not-nulled reference where `== null`
        /// returns true via Unity's operator overload but ReferenceEquals(_, null) returns false,
        /// causing the resurrection loop to silently skip).
        /// </summary>
        internal static volatile bool s_rootJustDestroyed;

        /// <summary>
        /// Iter-144 #543 fix: Set when RuntimeDriver.OnDestroy fires during a scene transition so
        /// AssetSwapSystem.OnDestroy can skip its bundle-unload (bundles must survive scene
        /// transitions; unloading mid-swap orphans chicken-sprite placeholders).
        /// </summary>
        internal static volatile bool s_skipBundleUnload;

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

            // DINOForge platform settings (exposed in BepInEx ConfigurationManager)
            ConfigEntry<bool> showOverlayOnStart = Config.Bind("General", "ShowDebugOverlayOnStart", false,
                "Show F9 debug overlay automatically when the game starts");
            ConfigEntry<bool> enableHotReload = Config.Bind("General", "EnableHotReload", true,
                "Watch pack files for changes and reload automatically (15s debounce)");
            ConfigEntry<int> hmrDebounceMs = Config.Bind("General", "HotReloadDebounceMs", 15000,
                new ConfigDescription("Milliseconds to wait after a file change before triggering reload",
                    new AcceptableValueRange<int>(500, 60000)));
            ConfigEntry<string> logLevel = Config.Bind("General", "LogLevel", "Info",
                new ConfigDescription("Logging verbosity for DINOForge runtime",
                    new AcceptableValueList<string>("Debug", "Info", "Warning", "Error")));

            // Realistic-GFX mode (Tier-B Phase-1 PoC). OFF by default. When "High", DINOForge injects a
            // URP post-processing Volume (ACES tonemap + bloom + color grading + vignette) onto the
            // active camera for a more cinematic, less TABS-flat look. See Graphics/GraphicsMode.cs and
            // docs/sessions/realistic-gfx-mode-rnd-20260530.md.
            ConfigEntry<string> graphicsTier = Config.Bind("Graphics", "Tier", "Vanilla",
                new ConfigDescription("Visual fidelity tier: Vanilla (no change) or High (cinematic post-processing).",
                    new AcceptableValueList<string>("Vanilla", "High")));
            _graphicsTier = graphicsTier;

            _showOverlayOnStart = showOverlayOnStart;
            _enableHotReload = enableHotReload;
            _hmrDebounceMs = hmrDebounceMs;

            // Session recorder (#971): record a REAL user playthrough (pointer + key + EventSystem
            // widget + screen frames) for in-process replay (#972) and journey embeds (#966).
            ConfigEntry<bool> recorderEnabled = Config.Bind("SessionRecorder", "Enabled", true,
                "Enable the F11 session recorder (records real user input + frames for replay/vision-verify)");
            ConfigEntry<int> recorderFrameMs = Config.Bind("SessionRecorder", "FrameIntervalMs", 500,
                new ConfigDescription("Periodic screen-frame cadence while recording (ms)",
                    new AcceptableValueRange<int>(100, 5000)));
            ConfigEntry<bool> recorderPerEvent = Config.Bind("SessionRecorder", "CaptureFramePerEvent", true,
                "Also capture a screen frame on every pointer down/up event");

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

            // Harmony — apply patches from this assembly
            // ModsButtonTextPatch (UI/UiGridHarmonyPatch.cs) intercepts Text/TMP_Text setters
            // to prevent DINO's UiGrid from overwriting our repurposed Mods button label.
            try
            {
                _harmony = new Harmony(PluginInfo.GUID);
                Bridge.DestroyGuardPatch.Apply(_harmony);

                // iter-149c: The iter-144 H7/H8/H9 DIAGNOSTIC probes (Resources.UnloadUnusedAssets,
                // AssetBundle.Unload/LoadFromFile, SceneManager.UnloadSceneAsync, World.Dispose) are
                // Harmony Prefix/Postfix patches on dispose/unload/teardown hot paths. Each prefix calls
                // `new StackTrace()` + synchronous BepInEx logging INSIDE those native calls. During the
                // InitialGameLoader->MainMenu transition, Unity.Entities.World.Dispose() tears down the
                // 45K-entity Default World while Mono is in teardown; a synchronous StackTrace+log there
                // contends the BepInEx log lock / blocks the managed plugin thread mid-dispose — exactly
                // matching the observed wedge (BepInEx's own LogOutput.log freezes at the same instant,
                // recurrence of the iter-144 mono_jit_cleanup gray-freeze). These probes are
                // diagnostics ONLY (no load-bearing functionality) — gate them OFF to test whether the
                // diagnostic probes are themselves causing the World.Dispose wedge. Files are kept intact
                // so the probes can be re-enabled for future native diagnosis. DestroyGuardPatch
                // (protects DINOForge_Root) and ModsButtonTextPatch (engine-UI label) stay ACTIVE.
                const bool EnableDisposeProbes = false;
#pragma warning disable CS0162 // unreachable code (intentional compile-time probe gate)
                if (EnableDisposeProbes)
                {
                    Bridge.ResourcesUnloadGuardPatch.Apply(_harmony);
                    Bridge.AssetBundleUnloadGuardPatch.Apply(_harmony);
                    Bridge.AssetBundleLoadGuardPatch.Apply(_harmony);
                    Bridge.SceneUnloadGuardPatch.Apply(_harmony);
                    Bridge.WorldDisposeGuardPatch.Apply(_harmony);
                }
#pragma warning restore CS0162

                UI.ModsButtonTextPatch.Apply(_harmony);
                Log.LogInfo($"Harmony initialized and patches applied (disposeProbes={EnableDisposeProbes}).");
            }
            catch (Exception ex)
            {
                Log.LogError($"Harmony init/patch failed: {ex}");
            }

            StartCoroutine(DeferredAwake());

            // Create a dedicated persistent GameObject that won't be destroyed.
            // The BepInEx-managed gameObject gets cleaned up during DINO's scene
            // transitions. A separate object with HideAndDontSave survives.
            try
            {
                PersistentRoot = new GameObject("DINOForge_Root");
                PersistentRoot.hideFlags = HideFlags.HideAndDontSave;
                UnityEngine.Object.DontDestroyOnLoad(PersistentRoot);
                Log.LogInfo("[Plugin] Persistent root GameObject created.");

                // EPIC-027: create the themed loading screen as EARLY as possible so it is
                // visible across the game's own InitialGameLoader asset-load window (before
                // RuntimeDriver finishes pack loading). It is faded out on pack-load complete /
                // world-ready / MainMenu. Created here (Awake, main thread) rather than waiting
                // for RuntimeDriver.Initialize's coroutine.
                try
                {
                    string lsPacksDir = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_packs");
                    UI.LoadingScreenController.Create(PersistentRoot, lsPacksDir, Logger);
                }
                catch (Exception ex)
                {
                    Log.LogWarning($"[Plugin] Early LoadingScreenController.Create failed (non-fatal): {ex.Message}");
                }
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

            // Realistic-GFX mode (Tier-B Phase-1 PoC). Attach the GraphicsMode component to the
            // persistent root, seed it from config, and re-apply on every scene change (Camera.main is
            // not available until a gameplay/menu scene loads, and DINO's PlayerLoop means we can't rely
            // on Update()). Inert unless Graphics.Tier == "High".
            try
            {
                Graphics.GraphicsMode gfx = PersistentRoot.AddComponent<Graphics.GraphicsMode>();
                gfx.ConfiguredTier = string.Equals(_graphicsTier?.Value, "High", StringComparison.OrdinalIgnoreCase)
                    ? Graphics.GraphicsTier.High
                    : Graphics.GraphicsTier.Vanilla;
                _graphicsMode = gfx;
                SceneManager.activeSceneChanged += (_, __) => _graphicsMode?.Apply();
                gfx.Apply();
                Log.LogInfo($"[Plugin] GraphicsMode attached (tier={gfx.ConfiguredTier}).");
            }
            catch (Exception ex)
            {
                Log.LogWarning($"[Plugin] GraphicsMode setup failed (non-fatal): {ex.Message}");
            }

            // Capture state for static resurrection callback (kept for emergency use)
            _resurrectionLog = Logger;
            _resurrectionConfig = Config;
            _resurrectionDump = dumpOnStartup.Value;
            _resurrectionDumpPath = dumpOutputPath.Value;

            StartResurrectionWatcher();

            // SPEC-004 Path 2: PlayerLoop.Update injection (preferred F9/F10 path at main menu).
            bool playerLoopInjected = false;
            try
            {
                playerLoopInjected = InjectPlayerLoopUpdate();
            }
            catch (Exception ex)
            {
                Log.LogWarning($"[Plugin] InjectPlayerLoopUpdate failed: {ex}");
            }

            // Win32 background poll only when PlayerLoop injection failed — both paths use
            // independent edge detection and would double-toggle F9/F10 if both run.
            if (!playerLoopInjected)
            {
                try
                {
                    Bridge.KeyInputSystem.StartKeyPollThread();
                    Log.LogInfo("[Plugin] PlayerLoop injection failed; using background key poll for F9/F10.");
                }
                catch (Exception ex)
                {
                    Log.LogWarning($"[Plugin] StartKeyPollThread failed: {ex}");
                }
            }

            // Session recorder (#971): F11 toggles recording of the real user playthrough.
            // Uses its own PlayerLoop sampler + Win32 F11 bg thread (independent of F9/F10).
            try
            {
                Capture.SessionRecorder.Configure(recorderEnabled.Value, recorderFrameMs.Value, recorderPerEvent.Value);
                Capture.SessionRecorder.Initialize();
            }
            catch (Exception ex)
            {
                Log.LogWarning($"[Plugin] SessionRecorder init failed: {ex}");
            }

            DebugLog.Write("Plugin", "Awake completed");
            Log.LogInfo("DINOForge Runtime loaded successfully.");
            Log.LogInfo("[DINOForge] Plugin.Awake() EXIT");
        }

        /// <summary>
        /// Defers ECS type discovery until after the first Unity frame so the loading
        /// screen can dismiss before the diagnostic walk starts.
        /// </summary>
        private IEnumerator DeferredAwake()
        {
            yield return null;

            try
            {
                Bridge.EcsTypeDiscovery.DiscoverAndLog();
                Log.LogInfo("[Plugin] ECS type discovery complete - check dinoforge_debug.log for details");
            }
            catch (Exception ex)
            {
                Log.LogWarning($"[Plugin] ECS type discovery failed: {ex}");
            }
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
            // Blocker 2 keystone fix (iter-149b, 2026-05-29): also subscribe to sceneLoaded.
            // Live log evidence (dinoforge_debug.log, all relaunches) shows activeSceneChanged
            // fires ONLY for '' and 'InitialGameLoader' — it NEVER fired for MainMenu. DINO loads
            // MainMenu ADDITIVELY (LoadSceneMode.Additive) or via an async path that does NOT change
            // the ACTIVE scene, so activeSceneChanged is silent for it while sceneLoaded DOES fire
            // for additive loads. With both the bg fallback wedged (Blocker 1) and no MainMenu
            // activeSceneChanged, resurrection never ran on a main thread. sceneLoaded is the missing
            // main-thread hook for the MainMenu activation. Both events run on the Unity main thread,
            // so resurrection (Unity ECalls) is safe in either handler.
            SceneManager.sceneLoaded += OnSceneLoaded;
            DebugLog.Write("Plugin", "[Plugin] activeSceneChanged + sceneLoaded watchers registered (iter-149b Blocker 2 fix).");
            StartResurrectionFallbackThread();
            // iter-149d BISECT (2026-05-29): PipeKeepAlive is the suspected NEW un-interruptible
            // waiter behind the recurring gray-freeze. PipeKeepAliveLoop polls EnsureServerAlive()
            // every 1s on a BACKGROUND thread; EnsureServerAlive does a pipe Stop()->Start() whenever
            // the bridge server thread is dead — which is ALWAYS the case immediately after
            // RuntimeDriver.OnDestroy calls RequestShutdown(). So during teardown OnDestroy disposes
            // the pipe handle (iter-144 fix) to unwedge the accept thread, but PipeKeepAlive instantly
            // re-creates a NamedPipeServerStream + re-arms BeginWaitForConnection — re-establishing
            // exactly the kernel ConnectNamedPipe wait that RequestShutdown just tore down. That
            // re-armed wait becomes the un-interruptible waiter that wedges mono_jit_cleanup during
            // World.Dispose. Gated OFF to isolate. The pipe must stay DOWN through teardown and only
            // be rebuilt by a clean main-thread resurrection (PlayerLoop/sceneLoaded path).
            const bool EnablePipeKeepAlive = false;
#pragma warning disable CS0162 // Unreachable code — intentional bisect gate (iter-149d).
            if (EnablePipeKeepAlive)
            {
                StartPipeKeepAliveThread();
            }
            else
            {
                DebugLog.Write("Plugin", "[Plugin] PipeKeepAlive thread DISABLED (iter-149d bisect: suspected re-arm of ConnectNamedPipe wedge during World.Dispose).");
            }
#pragma warning restore CS0162
        }

        // Blocker 2 keystone fix (iter-149b): sceneLoaded fires for additive scene loads where
        // activeSceneChanged stays silent (confirmed: MainMenu emitted NO activeSceneChanged).
        // Runs on the Unity main thread, so it is a safe place to perform resurrection. We log the
        // scene name + buildIndex + load mode on EVERY scene event so DINO's actual MainMenu emission
        // is observable, then drive the same main-thread revive path as OnActiveSceneChanged.
        private static void OnSceneLoaded(Scene scene, LoadSceneMode mode)
        {
            DebugLog.Write("Plugin", $"[Plugin] OnSceneLoaded: name='{scene.name}' buildIndex={scene.buildIndex} mode={mode} isLoaded={scene.isLoaded}");

            // Always remember the latest scene name so a fallback revive has a meaningful label.
            if (!string.IsNullOrEmpty(scene.name))
            {
                LastSceneNameForResurrection = scene.name;
            }

            // iter-149e ROOT-CAUSE fix: REVIVE FIRST. Previously EnsureEventSystemAlive() (a heavy
            // FindObjectsOfType ECall) + RecreateInCurrentWorld() ran BEFORE the revive; if either
            // wedged or threw during the MainMenu additive asset load, the revive never executed and
            // the plugin stayed dormant (the MDMP symptom). The revive is the load-bearing action —
            // run it on the main thread first, then do the (now best-effort) EventSystem/world fixups.
            MainThreadReviveIfNeeded(scene.name, "sceneLoaded(main-thread)");

            try { EnsureEventSystemAlive(); }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnSceneLoaded EnsureEventSystemAlive failed (non-fatal): {ex.Message}"); }
            try { Bridge.KeyInputSystem.RecreateInCurrentWorld(); }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnSceneLoaded RecreateInCurrentWorld failed (non-fatal): {ex.Message}"); }

            // EPIC-027: DINO loads MainMenu ADDITIVELY — activeSceneChanged stays silent for it, so
            // the MainMenu fade-out must also fire from sceneLoaded (the missing main-thread hook).
            try
            {
                if (scene.name == "MainMenu")
                    UI.LoadingScreenController.Instance?.BeginFadeOut();
            }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnSceneLoaded LoadingScreen fade failed (non-fatal): {ex.Message}"); }
        }

        /// <summary>
        /// Blocker 2 fix (iter-149b): shared main-thread revive entry point used by BOTH
        /// activeSceneChanged and sceneLoaded. Performs the actual resurrection on the Unity main
        /// thread (where Camera.main / AddComponent / Initialize ECalls are safe), then clears the
        /// need flags only on confirmed success. The bg fallback thread only MARKS the need; this is
        /// where the revive actually executes. Never throws to the Unity caller (Pattern #104/#111).
        /// </summary>
        private static void MainThreadReviveIfNeeded(string sceneName, string trigger)
        {
            // iter-149e: engine-driven heartbeat — this runs on the Unity main thread for EVERY
            // scene event, so it is a reliable liveness pulse that survives plugin-log freezes.
            BumpEngineHeartbeat(trigger);

            bool rootIsRefNull = ReferenceEquals(PersistentRoot, null);
            bool needsRevive = NeedsResurrection || NeedsDeferredResurrection || s_rootJustDestroyed || rootIsRefNull || PersistentRoot == null;
            if (!needsRevive)
            {
                return;
            }

            // iter-149e ROOT-CAUSE fix: a NEW scene event is a fresh opportunity to revive. During
            // InitialGameLoader (no Camera, no MainMenu) the create-root path can burn through
            // MaxResurrectionAttempts (3) and PERMANENTLY halt (IsResurrectionCapExhausted latches
            // true forever). When MainMenu later loads with a valid Camera, resurrection would stay
            // capped-out and never fire — the dormant-plugin symptom. Reset the consecutive-attempt
            // counter on each main-thread scene event so a loader-phase exhaustion never poisons the
            // MainMenu revive. The cap still bounds churn WITHIN a single scene's tick window.
            _resurrectionAttempts = 0;

            LastSceneNameForResurrection = string.IsNullOrEmpty(sceneName) ? LastSceneNameForResurrection : sceneName;
            NeedsDeferredResurrection = true;
            DebugLog.Write("Plugin", $"[Plugin] MainThreadReviveIfNeeded ({trigger}): revive needed (NeedsRes={NeedsResurrection} NeedsDefRes={NeedsDeferredResurrection} rootJustDestroyed={s_rootJustDestroyed} refNull={rootIsRefNull}) — invoking TryResurrect.");
            try
            {
                TryResurrect(LastSceneNameForResurrection ?? sceneName ?? "main-thread-unknown", trigger);
                if (ResurrectionSucceeded())
                {
                    NeedsResurrection = false;
                    NeedsDeferredResurrection = false;
                    s_rootJustDestroyed = false;
                    s_skipBundleUnload = false;
                    ResetGraceDeadline();
                    DebugLog.Write("Plugin", $"[Plugin] Resurrection complete via {trigger} (driver live; flags cleared).");
                }
                else
                {
                    DebugLog.Write("Plugin", $"[Plugin] {trigger} TryResurrect did not bring driver live — retained for next main-thread tick.");
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[Plugin] {trigger} TryResurrect threw: {ex.GetType().Name}: {ex.Message} — retained for next main-thread tick.");
            }
        }

        // Blocker 1 fix (iter-149b): dedicated pipe-keepalive thread. Pipe Stop()/Start() may block
        // (kernel-mode pipe teardown during asset loads) — running it here keeps that blocking work
        // OFF the resurrection fallback thread, so resurrection heartbeats keep ticking regardless of
        // pipe I/O latency. Polls every 1s; restart is idempotent (EnsureServerAlive no-ops when alive).
        private static Thread? _pipeKeepAliveThread;
        internal static readonly ManualResetEventSlim _pipeKeepAliveStopEvent = new(false);

        private static void StartPipeKeepAliveThread()
        {
            if (_pipeKeepAliveThread != null) return;
            _pipeKeepAliveThread = new Thread(PipeKeepAliveLoop)
            {
                Name = "DINOForge.PipeKeepAlive",
                IsBackground = true,
            };
            _pipeKeepAliveThread.Start();
            DebugLog.Write("Plugin", "[Plugin] Pipe-keepalive thread started (Blocker 1: pipe I/O off the resurrection heartbeat).");
        }

        private static void PipeKeepAliveLoop()
        {
            const int PipePollIntervalMs = 1000;
            DebugLog.Write("Plugin", "[Plugin] PipeKeepAlive: loop entered.");
            while (!_resurrectionFallbackStop)
            {
                try
                {
#pragma warning disable DF0116 // Intentional cooperative-stop blocking wait on the pipe-keepalive thread.
                    if (_pipeKeepAliveStopEvent.Wait(PipePollIntervalMs)) break;
#pragma warning restore DF0116
                    // This MAY block on a dead-pipe Stop()->Start(); that is acceptable here because
                    // it does not run on the resurrection heartbeat thread or the Unity main thread.
                    SharedBridgeServer?.EnsureServerAlive();
                }
                catch (ThreadAbortException)
                {
                    break;
                }
                catch (Exception ex)
                {
                    DebugLog.Write("Plugin", $"[Plugin] PipeKeepAlive EnsureServerAlive: {ex.Message}");
                }
            }
            DebugLog.Write("Plugin", "[Plugin] PipeKeepAlive thread exiting.");
        }

        private static void OnActiveSceneChanged(Scene oldScene, Scene newScene)
        {
            DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged: old='{oldScene.name}' new='{newScene.name}'");

            // iter-149e ROOT-CAUSE fix: REVIVE FIRST (mirrors OnSceneLoaded). The revive is the
            // load-bearing action and must not be gated behind the heavy EventSystem/world fixups
            // that can wedge during an asset load. activeSceneChanged fires on the Unity main thread.
            if (!string.IsNullOrEmpty(newScene.name))
            {
                LastSceneNameForResurrection = newScene.name;
            }
            MainThreadReviveIfNeeded(newScene.name, "activeSceneChanged(main-thread)");

            // Iter-144 menu-unclickable fix: DINO's MainMenu scene EventSystem is destroyed on
            // scene transitions, leaving EventSystem.current = null even though our
            // DontDestroyOnLoad'd EventSystem (DFCanvas) still exists. Re-promote (or recreate)
            // on every scene change so NativeMenuInjector clicks route correctly.
            try { EnsureEventSystemAlive(); }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged EnsureEventSystemAlive failed (non-fatal): {ex.Message}"); }
            try { Bridge.KeyInputSystem.RecreateInCurrentWorld(); }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged RecreateInCurrentWorld failed (non-fatal): {ex.Message}"); }

            // EPIC-027 loading-screen takeover: show during the game's own asset-load window
            // (InitialGameLoader / first scene), hide once the MainMenu is active.
            try
            {
                var ls = UI.LoadingScreenController.Instance;
                if (ls != null)
                {
                    if (newScene.name == "InitialGameLoader" || string.IsNullOrEmpty(oldScene.name))
                        ls.EnsureVisible();
                    else if (newScene.name == "MainMenu")
                        ls.BeginFadeOut();
                }
            }
            catch (Exception ex) { DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged LoadingScreen toggle failed (non-fatal): {ex.Message}"); }
            // Revive already executed at the TOP of this handler (iter-149e reorder).
        }

        /// <summary>
        /// Iter-144 menu-unclickable fix. DINO's MainMenu-scene EventSystem is destroyed during
        /// scene transitions, resetting <c>EventSystem.current</c> to null even when our
        /// DontDestroyOnLoad EventSystem (created by DFCanvas) is still alive in the hierarchy.
        /// Idempotent: re-promotes an existing one if found, otherwise creates a new
        /// DontDestroyOnLoad EventSystem with StandaloneInputModule.
        /// </summary>
        internal static void EnsureEventSystemAlive()
        {
            try
            {
                UnityEngine.EventSystems.EventSystem[] existing = UnityEngine.Object.FindObjectsOfType<UnityEngine.EventSystems.EventSystem>();
                UnityEngine.EventSystems.EventSystem? preferred = null;
                int activeCount = 0;
                string[] names = new string[existing.Length];

                for (int i = 0; i < existing.Length; i++)
                {
                    UnityEngine.EventSystems.EventSystem? system = existing[i];
                    if (system == null)
                    {
                        names[i] = "NULL";
                        continue;
                    }

                    names[i] = system.gameObject.name;
                    if (system.enabled) activeCount++;
                    if (preferred == null && IsDinoForgeEventSystem(system))
                    {
                        preferred = system;
                    }
                }

                if (preferred == null)
                {
                    if (UnityEngine.EventSystems.EventSystem.current != null &&
                        IsDinoForgeEventSystem(UnityEngine.EventSystems.EventSystem.current))
                    {
                        preferred = UnityEngine.EventSystems.EventSystem.current;
                    }
                }

                if (preferred == null)
                {
                    // None at all — create the authoritative DINOForge EventSystem.
                    var go = new GameObject("DINOForge_EventSystem_Restored");
                    UnityEngine.Object.DontDestroyOnLoad(go);
                    preferred = go.AddComponent<UnityEngine.EventSystems.EventSystem>();
                    go.AddComponent<UnityEngine.EventSystems.StandaloneInputModule>();
                    DebugLog.Write("Plugin", "[EventSystem] no scene EventSystem found — created DINOForge_EventSystem_Restored.");
                    existing = UnityEngine.Object.FindObjectsOfType<UnityEngine.EventSystems.EventSystem>();
                    names = new string[existing.Length];
                    for (int i = 0; i < existing.Length; i++)
                    {
                        names[i] = existing[i] != null ? existing[i].gameObject.name : "NULL";
                    }
                }
                else if (preferred.GetComponent<UnityEngine.EventSystems.StandaloneInputModule>() == null)
                {
                    preferred.gameObject.AddComponent<UnityEngine.EventSystems.StandaloneInputModule>();
                }

                if (!preferred.enabled)
                {
                    preferred.enabled = true;
                }

                for (int i = 0; i < existing.Length; i++)
                {
                    UnityEngine.EventSystems.EventSystem? system = existing[i];
                    if (system == null || ReferenceEquals(system, preferred))
                    {
                        continue;
                    }

                    if (system.enabled)
                    {
                        system.enabled = false;
                    }
                }

                if (!ReferenceEquals(UnityEngine.EventSystems.EventSystem.current, preferred))
                {
                    UnityEngine.EventSystems.EventSystem.current = preferred;
                }

                activeCount = 0;
                for (int i = 0; i < existing.Length; i++)
                {
                    UnityEngine.EventSystems.EventSystem? system = existing[i];
                    if (system != null && system.enabled)
                    {
                        activeCount++;
                    }
                }

                string currentName = UnityEngine.EventSystems.EventSystem.current != null
                    ? UnityEngine.EventSystems.EventSystem.current.gameObject.name
                    : "NULL";
                string key = $"{preferred.gameObject.name}|{currentName}|{existing.Length}|{activeCount}";
                if (key != _lastEventSystemReconcileKey)
                {
                    _lastEventSystemReconcileKey = key;
                    DebugLog.Write("Plugin", $"[EventSystem] reconcile: preferred={preferred.gameObject.name}, current={currentName}, total={existing.Length}, enabled={activeCount}, systems=[{string.Join(", ", names)}]");
                }
            }
            catch (Exception ex)
            {
                try { DebugLog.Write("Plugin", $"[EventSystem] ensure failed: {ex.GetType().Name}: {ex.Message}"); } catch { /* safe-swallow */ }
            }
        }

        private static bool IsDinoForgeEventSystem(UnityEngine.EventSystems.EventSystem system)
        {
            return system != null &&
                system.gameObject.name.StartsWith("DINOForge_", StringComparison.Ordinal);
        }

        // Iter-144 #546 fallback: Win32 background thread independent of any MonoBehaviour.
        // Survives RuntimeDriver destruction (the MB-owned background poll thread dies with its host).
        // Polls NeedsResurrection every 500ms; if set and no scene event has cleared it within the
        // grace window, attempts TryResurrect directly. Plugin class is referenced as long as the
        // BepInEx assembly is loaded, so this thread persists across scene transitions.
        private static Thread? _resurrectionFallbackThread;
#pragma warning disable CS0649 // Intentional shared shutdown flag for the fallback thread; set via runtime teardown path.
        private static volatile bool _resurrectionFallbackStop;
#pragma warning restore CS0649
        // P2 #879 Pattern #113 fix: ManualResetEventSlim allows the fallback loop to wake
        // immediately on shutdown instead of waiting out a full 500ms Thread.Sleep tick.
        // Mirrors _backgroundPollStopEvent pattern (#873).
        internal static readonly ManualResetEventSlim _resurrectionFallbackStopEvent = new(false);

        // FailureMode B fix (iter-149, 2026-05-29): the grace deadline MUST persist across loop
        // restarts. Previously `lastNeedsObservedUtc` was a LOCAL inside ResurrectionFallbackLoop;
        // when the loop re-entered (a new thread start, or a fresh "loop entered" after the prior
        // instance exited), the timer reset to MinValue and the 4000ms grace window NEVER elapsed,
        // so TryResurrect was detected every cycle but never executed. Latching the deadline as a
        // STATIC field means any loop iteration — even a brand-new one — honors the in-progress
        // grace window set by a prior iteration. DateTime.MinValue = "not armed".
        // Sentinel: DateTime.MinValue means no grace window is currently armed.
        private static DateTime _graceDeadlineUtc = DateTime.MinValue;
        private static readonly object _graceDeadlineLock = new();

        private static void StartResurrectionFallbackThread()
        {
            if (_resurrectionFallbackThread != null) return;
            _resurrectionFallbackThread = new Thread(ResurrectionFallbackLoop)
            {
                Name = "DINOForge.ResurrectionFallback",
                IsBackground = true,
            };
            _resurrectionFallbackThread.Start();
            DebugLog.Write("Plugin", "[Plugin] Resurrection fallback thread started.");
        }

        private static void ResurrectionFallbackLoop()
        {
            long iterationCount = 0;
            const int PollIntervalMs = 500;
            const int GraceWindowMs = 4000; // 4s after NeedsResurrection observed, attempt direct revive
            // Iter-144 #547 H6: 4-iter (2s) heartbeat — frequent enough to distinguish "Mono wedged"
            // from "no scene events firing yet" in the post-OnDestroy gray-freeze window. Previous 10s
            // cadence left ambiguous gaps where probe timing missed the window entirely.
            const int HeartbeatEveryNIterations = 4;
            DebugLog.Write("Plugin", "[Plugin] ResurrectionFallback: loop entered.");
            while (!_resurrectionFallbackStop)
            {
                try
                {
                    // P2 #879 Pattern #113 fix: cancellation-aware wait instead of Thread.Sleep.
#pragma warning disable DF0116 // Intentional blocking wait on the fallback thread's cooperative stop event.
                    if (_resurrectionFallbackStopEvent.Wait(PollIntervalMs)) break;
#pragma warning restore DF0116
                    iterationCount++;

                    // iter-149e: bump the engine heartbeat file from the FALLBACK thread too. This is
                    // a separate file from the debug log, so if the heartbeat counter keeps advancing
                    // with source "fallback" while the plugin LOG is frozen, the bg thread is ALIVE and
                    // the freeze is purely a log-write contention — vs the counter freezing too, which
                    // proves the bg thread itself is suspended/dead (the WinDbg dormant-plugin case).
                    BumpEngineHeartbeat("fallback#" + iterationCount);

                    // Blocker 1 fix (iter-149b, 2026-05-29): DO NOT call EnsureServerAlive() here.
                    // EnsureServerAlive performs a pipe Stop()->Start() (NamedPipeServerStream dispose +
                    // fresh server thread) whenever BridgeServerThreadAlive=False — which is ALWAYS the
                    // case right after RuntimeDriver.OnDestroy's RequestShutdown(). That pipe
                    // teardown/recreate BLOCKS this background thread during the
                    // InitialGameLoader->MainMenu asset-load window. Confirmed in dinoforge_debug.log:
                    // heartbeats #4..#20 tick cleanly until OnDestroy, then heartbeat #24 NEVER appears
                    // (the loop wedged on the pipe restart), so the grace-windowed revive is never
                    // reached. The deadlock did not disappear when TryResurrect was removed from this
                    // loop in 6be0f5e3 — it MOVED to the pipe restart on this same background thread.
                    //
                    // The fallback loop's PRIMARY job is the grace-windowed revive heartbeat; pipe I/O
                    // must NEVER starve it. Pipe keepalive is now owned by:
                    //   (a) DINOForgePlayerLoopUpdate (main thread, %60 gate) -> EnsureServerAlive(), and
                    //   (b) a dedicated background pipe-keepalive thread (PipeKeepAliveLoop) which may
                    //       block freely on Stop()/Start() without affecting resurrection heartbeats.
                    // This loop now performs pure managed work only (flag checks + grace timer + MARK).

                    // Iter-144 #547 H5: emit periodic heartbeat to prove Mono runtime + this thread are alive.
                    // If the gray-freeze is a native deadlock at runtime level, heartbeats stop appearing
                    // immediately after OnDestroy. If they keep appearing, the hang is elsewhere.
                    if (iterationCount % HeartbeatEveryNIterations == 0)
                    {
                        DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback heartbeat #{iterationCount} NeedsRes={NeedsResurrection} NeedsDefRes={NeedsDeferredResurrection} rootNull={PersistentRoot == null}");
                    }
                    // Iter-144 #543 fix: OR in s_rootJustDestroyed flag — when RuntimeDriver.OnDestroy
                    // fires, PersistentRoot may hold a destroyed-but-not-nulled Unity fake-null reference,
                    // and NeedsResurrection could be cleared by a stale scene event before the new
                    // driver attaches. The companion flag is the source of truth for "RuntimeDriver
                    // died and has not been replaced yet."
                    bool needsRevive = NeedsResurrection || NeedsDeferredResurrection || s_rootJustDestroyed;
                    if (!needsRevive)
                    {
                        // No need observed: disarm the (static) grace window so a future need starts fresh.
                        ResetGraceDeadline();
                        continue;
                    }

                    // FailureMode B fix: latch the grace DEADLINE in a STATIC field so a loop restart
                    // RESUMES the in-progress window instead of resetting it. Returns true only once
                    // the latched deadline has elapsed; until then we keep polling.
                    if (!IsGraceWindowElapsed(GraceWindowMs, out bool justArmed))
                    {
                        if (justArmed)
                        {
                            DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback: NeedsResurrection observed, grace deadline armed ({GraceWindowMs}ms).");
                        }
                        continue;
                    }

                    // Blocker 2 fix (iter-149b, 2026-05-29): the grace window has elapsed without a
                    // main-thread scene event resolving the revive. This BACKGROUND thread MUST NOT
                    // call TryResurrect directly — TryResurrect reaches Unity ECalls (Camera.main /
                    // AddComponent / RuntimeDriver.Initialize -> coroutine touching Resources/asset
                    // APIs) which DEADLOCK on a bg thread during the InitialGameLoader->MainMenu asset
                    // load (memory: "Resources.* from a bg thread DEADLOCKS during asset loading").
                    // The bg path's ONLY job is to keep the need MARKED so a main-thread consumer
                    // (DINOForgePlayerLoopUpdate or a scene event) performs the actual revive on a
                    // thread where Unity ECalls are safe. We re-arm and keep heart-beating so the need
                    // never silently drops, and so the heartbeat proves the loop is no longer wedged.
                    // iter-149e ROOT-CAUSE fix (WinDbg MDMP): the previous code called
                    // ResurrectionSucceeded() HERE — which performs a Unity ECall
                    // (PersistentRoot.GetComponent<RuntimeDriver>()) on THIS BACKGROUND THREAD.
                    // During the InitialGameLoader->MainMenu asset load, Unity ECalls from a bg
                    // thread wedge/tear the calling thread (memory: "Resources.* from a bg thread
                    // DEADLOCKS during asset loading"; GetComponent is in the same ECall family).
                    // The MDMP showed this fallback thread GONE post-OnDestroy with NO managed frame
                    // and NO stop-flag ever set — i.e. it was torn inside the ECall, never reaching
                    // heartbeat #12. The bg loop MUST do PURE managed work only: mark the need and
                    // re-arm. The actual revive (and the GetComponent liveness probe) happens ONLY on
                    // the Unity main thread via OnSceneLoaded/OnActiveSceneChanged -> MainThreadReviveIfNeeded.
                    NeedsDeferredResurrection = true;
                    RearmGraceDeadline(GraceWindowMs);
                    DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback: grace window {GraceWindowMs}ms elapsed — MARKED NeedsDeferredResurrection for main-thread revive (NO bg-thread Unity ECalls — iter-149e). scene='{LastSceneNameForResurrection ?? "fallback-unknown"}'.");
                }
                catch (ThreadAbortException)
                {
                    break;
                }
                catch (Exception ex)
                {
                    DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback loop error: {ex.Message}");
                }
            }
            DebugLog.Write("Plugin", "[Plugin] Resurrection fallback thread exiting.");
        }

        /// <summary>
        /// FailureMode B helper: returns true once the latched (static) grace deadline has elapsed.
        /// If no deadline is armed, arms one (now + graceWindowMs) and returns false with
        /// <paramref name="justArmed"/>=true. Survives loop restarts because the deadline is static.
        /// </summary>
        private static bool IsGraceWindowElapsed(int graceWindowMs, out bool justArmed)
        {
            justArmed = false;
            lock (_graceDeadlineLock)
            {
                if (_graceDeadlineUtc == DateTime.MinValue)
                {
                    _graceDeadlineUtc = DateTime.UtcNow.AddMilliseconds(graceWindowMs);
                    justArmed = true;
                    return false;
                }
                return DateTime.UtcNow >= _graceDeadlineUtc;
            }
        }

        /// <summary>Re-arms the static grace deadline to now + graceWindowMs (back-off after a failed/partial revive).</summary>
        private static void RearmGraceDeadline(int graceWindowMs)
        {
            lock (_graceDeadlineLock)
            {
                _graceDeadlineUtc = DateTime.UtcNow.AddMilliseconds(graceWindowMs);
            }
        }

        /// <summary>Disarms the static grace deadline (no resurrection currently needed, or revive succeeded).</summary>
        private static void ResetGraceDeadline()
        {
            lock (_graceDeadlineLock)
            {
                _graceDeadlineUtc = DateTime.MinValue;
            }
        }

        /// <summary>
        /// FailureMode B helper: true only when the resurrection actually brought a live, initialized
        /// RuntimeDriver online. Used to decide whether to clear NeedsResurrection (only on success)
        /// versus retain the need + re-arm for another attempt. Never throws to the caller.
        /// </summary>
        private static bool ResurrectionSucceeded()
        {
            try
            {
                if (ReferenceEquals(PersistentRoot, null)) return false;
                RuntimeDriver? driver = PersistentRoot!.GetComponent<RuntimeDriver>();
                return driver != null && driver.IsInitialized;
            }
            catch (Exception ex)
            {
                // Pattern #104/#111: surface, do not silently swallow.
                try { DebugLog.Write("Plugin", $"[Plugin] ResurrectionSucceeded check threw: {ex.GetType().Name}: {ex.Message}"); } catch { /* diagnostic only */ }
                return false;
            }
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
            DebugLog.Write("Plugin", $"[Plugin] MarkNeedsDeferredResurrection via {trigger}");
            NeedsDeferredResurrection = true;
        }

        internal static void TryResurrect(string sceneName, string trigger)
        {
            if (ReferenceEquals(PersistentRoot, null))
            {
                if (IsResurrectionCapExhausted())
                    return;

                _resurrectionAttempts++;
                try
                {
                    TryResurrectCreateRoot(sceneName, trigger);
                }
                catch (Exception)
                {
                    PersistentRoot = null;
                }
                return;
            }

            TryResurrectWhenRootAlive(trigger);
        }

        /// <summary>SPEC-004 KIS-NF4: pure C# cap gate — no Unity ECalls (unit-test safe).</summary>
        private static bool IsResurrectionCapExhausted()
        {
            if (_resurrectionAttempts < MaxResurrectionAttempts)
                return false;

            if (_resurrectionAttempts == MaxResurrectionAttempts)
            {
                try
                {
                    DebugLog.Write("Plugin", $"[Plugin] TryResurrect: giving up after {MaxResurrectionAttempts} consecutive failures — resurrection loop halted.");
                }
                catch { } // safe-swallow: diagnostic only; must not escape outside Unity player
                _resurrectionAttempts++;
            }

            return true;
        }

        private static void TryResurrectWhenRootAlive(string trigger)
        {
            try
            {
                _resurrectionAttempts = 0;
                NeedsResurrection = false;
                // Check if RuntimeDriver component exists and is initialized
                RuntimeDriver? existing = PersistentRoot!.GetComponent<RuntimeDriver>();
                if (existing != null && existing.IsInitialized)
                {
                    DebugLog.Write("Plugin", $"[Plugin] TryResurrect ({trigger}): RuntimeDriver already running, ensuring KeyInputSystem is registered...");
                    // CRITICAL: Always ensure KeyInputSystem is registered in the current world,
                    // even if RuntimeDriver is already initialized. Scene transitions may have
                    // created a new world that KeyInputSystem needs to be registered in.
                    Bridge.KeyInputSystem.RecreateInCurrentWorld();
                    return;
                }
                // RuntimeDriver exists but wasn't initialized — initialize it
                if (existing != null)
                {
                    DebugLog.Write("Plugin", $"[Plugin] TryResurrect ({trigger}): RuntimeDriver exists but not initialized, initializing...");
                    existing.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);
                    return;
                }
                // No RuntimeDriver component — create one
                DebugLog.Write("Plugin", $"[Plugin] TryResurrect ({trigger}): PersistentRoot exists but no RuntimeDriver, adding component...");
                RuntimeDriver driver = PersistentRoot!.AddComponent<RuntimeDriver>();
                driver.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);
            }
            catch (Exception ex)
            {
                try
                {
                    DebugLog.Write("Plugin", $"[Plugin] TryResurrectWhenRootAlive FAILED ({trigger}): {ex.Message}");
                }
                catch { } // safe-swallow: diagnostic only
            }
        }

        private static void TryResurrectCreateRoot(string sceneName, string trigger)
        {
            try
            {
                DebugLog.Write("Plugin", $"[Plugin] TryResurrect attempt {_resurrectionAttempts}/{MaxResurrectionAttempts} via {trigger} on '{sceneName}' — resurrecting...");
                // Try to attach RuntimeDriver to DINO's main camera — DINO never destroys its own camera
                Camera? cam = Camera.main ?? (Camera.allCameras.Length > 0 ? Camera.allCameras[0] : null);
                GameObject host;
                if (cam != null)
                {
                    host = cam.gameObject;
                    DebugLog.Write("Plugin", $"[Plugin] Attaching to existing camera '{host.name}'");
                }
                else
                {
                    // Fallback: create our own object
                    host = new GameObject("DINOForge_Root");
                    host.hideFlags = HideFlags.HideAndDontSave;
                    UnityEngine.Object.DontDestroyOnLoad(host);
                    DebugLog.Write("Plugin", $"[Plugin] No camera found, using new GameObject");
                }
                PersistentRoot = host;

                RuntimeDriver driver = host.AddComponent<RuntimeDriver>();
                driver.Initialize(_resurrectionLog!, _resurrectionConfig!, _resurrectionDump, _resurrectionDumpPath);

                // Immediately register KeyInputSystem in the current ECS world.
                // The polling thread will also do this, but scene transitions may have already
                // created a new DefaultGameObjectInjectionWorld that the thread hasn't caught yet.
                // This call bridges the gap so the pump is active without waiting for a poll cycle.
                Bridge.KeyInputSystem.RecreateInCurrentWorld();
                _resurrectionAttempts = 0;
                NeedsResurrection = false;
                DebugLog.Write("Plugin", $"[Plugin] Resurrection complete via {trigger} on '{sceneName}' host='{host.name}'.");
            }
            catch (Exception ex)
            {
                PersistentRoot = null;
                try
                {
                    DebugLog.Write("Plugin", $"[Plugin] Resurrection FAILED via {trigger}: {ex.Message}");
                }
                catch { } // safe-swallow: diagnostic only; must not escape and break KIS-NF4 cap semantics
            }
        }


        private static bool _playerLoopHarmonyPatched;

        /// <summary>SPEC-004 Path 2: append <see cref="Bridge.PlayerLoopKeyInputInjection.DINOForgeUpdateMarker"/> to PlayerLoop.Update.</summary>
        private static bool InjectPlayerLoopUpdate()
        {
            bool injected = Bridge.PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                typeof(Bridge.PlayerLoopKeyInputInjection.DINOForgeUpdateMarker),
                DINOForgePlayerLoopUpdate);
            if (injected)
            {
                PatchPlayerLoopRejection();
                Log?.LogInfo("[Plugin] PlayerLoop DINOForgeUpdate injected (SPEC-004 Path 2).");
            }
            else
            {
                Log?.LogWarning("[Plugin] PlayerLoop DINOForgeUpdate injection failed (Update subsystem missing?).");
            }

            return injected;
        }

        private static int _playerLoopEventSystemTick;
        private static string? _lastEventSystemReconcileKey;
        private static bool _prevF9;
        private static bool _prevF10;

        [System.Runtime.InteropServices.DllImport("user32.dll", EntryPoint = "GetAsyncKeyState")]
        private static extern short PluginGetAsyncKeyState(int vKey);

        private static void DINOForgePlayerLoopUpdate()
        {
            _playerLoopEventSystemTick++;
            if (_playerLoopEventSystemTick % 60 == 1)
            {
                // iter-149e: heartbeat from the PlayerLoop too. The WinDbg MDMP showed NO Harmony/
                // DINOForge frame on the idle main thread — i.e. this injected PlayerLoop callback
                // may NOT actually tick under DINO's replaced PlayerLoop. If dinoforge_heartbeat.txt
                // only ever advances with scene-event sources (never "playerloop"), that confirms
                // the PlayerLoop revive path is dead and scene events are the sole reliable hook.
                BumpEngineHeartbeat("playerloop");
                EnsureEventSystemAlive();
                try { SharedBridgeServer?.EnsureServerAlive(); }
                catch (Exception ex) { DebugLog.Write("Plugin", $"[PlayerLoop] EnsureServerAlive: {ex.Message}"); }

                // FailureMode B definitive fix (iter-149, 2026-05-29): MAIN-THREAD resurrection
                // consumer. The PlayerLoop Update injection runs on the Unity MAIN THREAD every
                // frame and SURVIVES RuntimeDriver teardown (it is static + Harmony-injected), so
                // it is the correct place to perform resurrection — TryResurrect's Unity ECalls
                // (Camera.main / AddComponent / Initialize) are main-thread-safe here, whereas the
                // ResurrectionFallback BACKGROUND thread deadlocks on the same calls (and even on
                // the Unity `==`/GetComponent ECalls) during the InitialGameLoader→MainMenu asset
                // load — which is why its heartbeat went silent and the driver never revived.
                // Throttled to once/sec (the %60 gate) so we don't thrash; idempotent + cap-guarded
                // inside TryResurrect. Need flags are cleared only on confirmed success.
                ConsumeResurrectionOnMainThread();
            }

            if (!System.Runtime.InteropServices.RuntimeInformation.IsOSPlatform(
                    System.Runtime.InteropServices.OSPlatform.Windows))
            {
                return;
            }

            const int VK_F9 = 0x78;
            const int VK_F10 = 0x79;
            const int KEY_PRESSED = unchecked((int)0x8000);

            bool f9Now = (PluginGetAsyncKeyState(VK_F9) & KEY_PRESSED) != 0;
            bool f10Now = (PluginGetAsyncKeyState(VK_F10) & KEY_PRESSED) != 0;

            if (f9Now && !_prevF9)
            {
                try { Bridge.KeyInputSystem.OnF9Pressed?.Invoke(); }
                catch (System.Exception ex) { DebugLog.Write("Plugin", $"[PlayerLoop] F9 handler threw: {ex.Message}"); }
            }
            if (f10Now && !_prevF10)
            {
                try { Bridge.KeyInputSystem.OnF10Pressed?.Invoke(); }
                catch (System.Exception ex) { DebugLog.Write("Plugin", $"[PlayerLoop] F10 handler threw: {ex.Message}"); }
            }

            _prevF9 = f9Now;
            _prevF10 = f10Now;
        }

        /// <summary>
        /// FailureMode B definitive fix (iter-149, 2026-05-29): MAIN-THREAD resurrection consumer.
        /// Invoked from <see cref="DINOForgePlayerLoopUpdate"/> (throttled by the caller's %60 gate),
        /// this runs on the Unity main thread and SURVIVES RuntimeDriver teardown (it is a static
        /// method reached via the Harmony-injected PlayerLoop entry). Unlike the ResurrectionFallback
        /// BACKGROUND thread — which deadlocks on Unity ECalls (Camera.main / AddComponent / Initialize
        /// touching Resources/asset APIs) during the InitialGameLoader→MainMenu asset load — every call
        /// made here is main-thread-safe.
        ///
        /// Idempotent and cap-guarded inside <see cref="TryResurrect"/>. Need flags are cleared only on
        /// confirmed success (<see cref="ResurrectionSucceeded"/>). Never throws to Unity (Pattern #104/#111).
        /// </summary>
        private static void ConsumeResurrectionOnMainThread()
        {
            try
            {
                bool needsRevive = NeedsResurrection || NeedsDeferredResurrection || s_rootJustDestroyed;
                if (!needsRevive)
                {
                    return;
                }

                // Cap gate: when PersistentRoot is gone, TryResurrect's create-root path is bounded by
                // MaxResurrectionAttempts. Checking here too avoids logging churn once the cap is hit.
                if (ReferenceEquals(PersistentRoot, null) && IsResurrectionCapExhausted())
                {
                    return;
                }

                string sceneName;
                try { sceneName = LastSceneNameForResurrection ?? SceneManager.GetActiveScene().name; }
                catch { sceneName = LastSceneNameForResurrection ?? "main-thread-unknown"; }

                DebugLog.Write("Plugin", $"[Plugin] ConsumeResurrectionOnMainThread: revive needed (NeedsRes={NeedsResurrection} NeedsDefRes={NeedsDeferredResurrection} rootJustDestroyed={s_rootJustDestroyed}) — invoking TryResurrect (scene='{sceneName}').");
                TryResurrect(sceneName, "main-thread-playerloop");

                if (ResurrectionSucceeded())
                {
                    NeedsResurrection = false;
                    NeedsDeferredResurrection = false;
                    s_rootJustDestroyed = false;
                    s_skipBundleUnload = false;
                    ResetGraceDeadline();
                    DebugLog.Write("Plugin", "[Plugin] Resurrection complete via main-thread-playerloop (driver live; flags cleared).");
                }
            }
            catch (Exception ex)
            {
                // Pattern #104/#111: surface, never throw into the PlayerLoop.
                try { DebugLog.Write("Plugin", $"[Plugin] ConsumeResurrectionOnMainThread threw: {ex.GetType().Name}: {ex.Message}"); } catch { /* diagnostic only */ }
            }
        }

        private static void PatchPlayerLoopRejection()
        {
            if (_playerLoopHarmonyPatched)
            {
                return;
            }

            try
            {
                var harmony = new Harmony("dinoforge.plugin.playerloop");
                System.Reflection.MethodInfo? original = typeof(PlayerLoop).GetMethod(
                    nameof(PlayerLoop.SetPlayerLoop),
                    System.Reflection.BindingFlags.Public | System.Reflection.BindingFlags.Static);
                if (original == null)
                {
                    Log?.LogWarning("[Plugin] PatchPlayerLoopRejection: SetPlayerLoop not found.");
                    return;
                }

                System.Reflection.MethodInfo? postfix = typeof(Plugin).GetMethod(
                    nameof(OnPlayerLoopSet),
                    System.Reflection.BindingFlags.NonPublic | System.Reflection.BindingFlags.Static);
                harmony.Patch(original, postfix: new HarmonyMethod(postfix));
                _playerLoopHarmonyPatched = true;
                DebugLog.Write("Plugin", "[Plugin] Harmony postfix on PlayerLoop.SetPlayerLoop applied.");
            }
            catch (Exception ex)
            {
                Log?.LogWarning($"[Plugin] PatchPlayerLoopRejection failed: {ex.Message}");
            }
        }

        private static void OnPlayerLoopSet()
        {
            // Blocker 2 diagnostic (iter-149b): log every PlayerLoop rebuild + whether re-injection
            // re-added our DINOForgeUpdateMarker. If DINO rebuilds the loop entering MainMenu and our
            // marker is dropped, this surfaces it. Even if re-injection fails, the sceneLoaded /
            // activeSceneChanged main-thread revive path (Blocker 2) covers resurrection, so the
            // engine UI no longer depends solely on the PlayerLoop marker surviving.
            Bridge.PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(() =>
                Bridge.PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                    typeof(Bridge.PlayerLoopKeyInputInjection.DINOForgeUpdateMarker),
                    DINOForgePlayerLoopUpdate));

            // iter-149e DECISIVE fix (WinDbg MDMP + live repro): after RuntimeDriver.OnDestroy on the
            // InitialGameLoader->MainMenu transition, ALL our managed activity halts — the
            // ResurrectionFallback bg thread stops heart-beating (it armed the grace window then went
            // silent), no MainMenu sceneLoaded/activeSceneChanged ever reaches our static handlers, and
            // the injected PlayerLoop callback never ticks. The engine stays healthy (process alive,
            // Responding=True, MainMenu rendered, engine heartbeat advances) — a DORMANT-PLUGIN bug,
            // not a native wedge. The ONE callback DINO itself drives on the main thread post-teardown
            // is THIS Harmony postfix on PlayerLoop.SetPlayerLoop — DINO calls SetPlayerLoop while
            // bringing up MainMenu systems. So drive the revive directly from HERE, on the main thread,
            // where TryResurrect's Unity ECalls (Camera.main / AddComponent / Initialize) are safe.
            // This does not depend on a post-teardown scene event or on our suspended bg threads.
            try { MainThreadReviveIfNeeded(LastSceneNameForResurrection ?? "playerloop-set", "playerloop-set(main-thread)"); }
            catch (Exception ex) { try { DebugLog.Write("Plugin", $"[Plugin] OnPlayerLoopSet revive threw: {ex.GetType().Name}: {ex.Message}"); } catch { /* diagnostic only */ } }

            try
            {
                bool markerPresent = Bridge.PlayerLoopKeyInputInjection.ContainsMarkerInUpdate(
                    UnityEngine.LowLevel.PlayerLoop.GetCurrentPlayerLoop(),
                    typeof(Bridge.PlayerLoopKeyInputInjection.DINOForgeUpdateMarker));
                DebugLog.Write("Plugin", $"[Plugin] OnPlayerLoopSet postfix fired — DINOForgeUpdateMarker re-injected={markerPresent}.");
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[Plugin] OnPlayerLoopSet marker-check threw: {ex.Message}");
            }
        }

        private static void LogInstallDiagnostics()
        {
            string loadedAssemblyPath = typeof(Plugin).Assembly.Location;
            string primaryRuntimePath = Path.Combine(Paths.PluginPath, "DINOForge.Runtime.dll");
            string legacyRuntimePath = Path.Combine(Paths.BepInExRootPath, "ecs_plugins", "DINOForge.Runtime.dll");
            string backupRuntimePath = Path.Combine(Paths.PluginPath, "DINOForge.Runtime.dll.bak");

            Log.LogInfo($"[Plugin] Loaded runtime assembly from: {loadedAssemblyPath}");
            DebugLog.Write("Plugin", $"[Plugin] Loaded runtime assembly from: {loadedAssemblyPath}");

            if (File.Exists(legacyRuntimePath))
            {
                string message = $"[Plugin] Legacy runtime copy detected at deprecated path: {legacyRuntimePath}";
                Log.LogWarning(message);
                DebugLog.Write("Plugin", message);
            }

            if (File.Exists(primaryRuntimePath) && File.Exists(legacyRuntimePath))
            {
                string message = $"[Plugin] Duplicate runtime assemblies detected. Primary='{primaryRuntimePath}', Legacy='{legacyRuntimePath}'";
                Log.LogWarning(message);
                DebugLog.Write("Plugin", message);
            }

            if (File.Exists(backupRuntimePath))
            {
                string message = $"[Plugin] Stale runtime backup file detected: {backupRuntimePath}";
                Log.LogWarning(message);
                DebugLog.Write("Plugin", message);
            }

            if (!string.Equals(loadedAssemblyPath, primaryRuntimePath, StringComparison.OrdinalIgnoreCase))
            {
                string message = $"[Plugin] Runtime loaded from non-canonical location. Expected '{primaryRuntimePath}', actual '{loadedAssemblyPath}'";
                Log.LogWarning(message);
                DebugLog.Write("Plugin", message);
            }
        }

        private void OnDestroy()
        {
            // The BepInEx-managed object is being destroyed (expected in DINO).
            // The persistent root and RuntimeDriver continue running independently.
            Log?.LogInfo("[Plugin] BepInEx plugin object OnDestroy (persistent root still alive).");
            try { _harmony?.UnpatchSelf(); } catch (Exception ex) { DebugLog.Write("Plugin", $"OnDestroy Harmony.UnpatchSelf failed: {ex.Message}"); }
            // P0 fix: stop the Win32 F9/F10 polling thread on plugin teardown.
            try { Bridge.KeyInputSystem.StopKeyPollThread(); } catch (Exception ex) { DebugLog.Write("Plugin", $"OnDestroy StopKeyPollThread failed: {ex.Message}"); }
            try { Capture.SessionRecorder.Shutdown(); } catch (Exception ex) { DebugLog.Write("Plugin", $"OnDestroy SessionRecorder.Shutdown failed: {ex.Message}"); }
            // Iter-144 #547 H5 gray-freeze fix: do NOT unsubscribe activeSceneChanged here.
            // The handler is a static method on the Plugin class; the static delegate survives
            // BepInEx Plugin instance destruction. Previously we unsubscribed here, breaking
            // resurrection on second-and-later scene transitions (only the Win32 fallback thread
            // could revive). Keeping the subscription live is the correct behavior — there's
            // no leak because the target is a static method.
            // Harmony unpatch is also deliberately skipped — runtime patches must persist across
            // BepInEx Plugin object death since the actual functionality lives on RuntimeDriver/
            // ModPlatform which outlive this BepInEx wrapper.
            DebugLog.Write("Plugin", "OnDestroy called (BepInEx object only); activeSceneChanged + fallback thread persist by design (iter-144 #547).");
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

        // EPIC-027: themed loading-screen takeover (shown during mod init / scene loads,
        // faded out when the target scene + engine UI are ready).
        private LoadingScreenController? _loadingScreen;

        // Active UI hosts.
        // _modMenuHost is always set to the active menu (UGUI when healthy, IMGUI fallback otherwise).
        // _debugOverlay is ALWAYS added (it owns the IMGUI F9 debug panel).
        private IModMenuHost? _modMenuHost;
        private IModSettingsHost? _modSettingsHost;
        private DebugOverlayBehaviour? _debugOverlay;
        private HudIndicator? _hudIndicator;
        private NativeMenuInjector? _nativeMenuInjector;
        private MainMenuThemer? _mainMenuThemer;
        private UI.CanvasReskinner? _canvasReskinner;
        private int _reskinRetryCount;

        // ── Engine-UI self-healing (fix/engine-ui-injection-race) ────────────────
        // RunMainMenuInit() is idempotent and re-runnable; these track its state so the
        // main-thread pump can bounded-retry injection until the MODS button exists, and so
        // the scene-change handler can re-run the menu-mode init when re-entering a menu scene.
        // This kills the intermittent "no Mods button / no engine UI" race: a single missed
        // timing window (ECS-world gate, late canvas, custom Selectable button) auto-recovers.
        private bool _engineUiHeartbeatLogged;
        private int _menuInitRetryFrames;
        // Bounded retry budget — re-attempt MODS injection for up to N pump frames after the
        // initial menu-mode init. At ~once-per-frame this covers several seconds of menu fade-in.
        private const int MenuInitMaxRetryFrames = 600;
        // Subscribed once; reset menu-mode init state when a menu scene becomes active again.
        private bool _sceneChangeSubscribed;

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
        private readonly object _deferredWorkLock = new();
        private bool _bootSequenceStarted;
        private bool _worldReadyProcessing;
        private World? _pendingWorldReady;
        private bool _hasPendingWorldReady;
        private bool _pendingPackReload;
        private string? _pendingPackReloadReason;
        private bool _pendingPackToggle;
        private string? _pendingPackToggleId;
        private bool _pendingPackToggleEnabled;
        private World? _pendingCatalogWorld;

        // HMR tiered reloader — created once ModPlatform is available.
        private HotReload.HmrTieredReloader? _hmrTieredReloader;

        // Profiles manager (#918) — created once BepInEx root path is known.
        private Profiles.ProfileManager? _profileManager;

        // ── Step 8: Update checker (#899) ─────────────────────────────────────────
        // The Task is fired on the thread pool after pack-load and polled in the
        // deferred-work coroutine loop. Results are pushed to the UI panel when ready.
        private System.Threading.Tasks.Task<System.Collections.Generic.IReadOnlyList<Updates.UpdateInfo>>? _updateCheckTask;
        private bool _updateCheckPushed;

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
            if (_bootSequenceStarted)
            {
                _log.LogWarning("[RuntimeDriver] Initialize() called after boot sequence already started.");
                return;
            }

            _bootSequenceStarted = true;
            StartCoroutine(InitializeRoutine());
        }

        private IEnumerator InitializeRoutine()
        {
            yield return null;

            RunPhaseWithAbortGuard("L10n.Initialize", () =>
            {
                try
                {
                    Localization.L10n.Initialize();
                    _log.LogInfo($"[RuntimeDriver] L10n initialized with locale: {Localization.L10n.CurrentLocale}");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] L10n initialization failed: {ex}");
                }
            });
            yield return null;

            RunPhaseWithAbortGuard("CleanupUiInterceptors", CleanupUiInterceptors);
            yield return null;

            RunPhaseWithAbortGuard("UiAssets.Initialize", () =>
            {
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
            });
            yield return null;

            RunPhaseWithAbortGuard("ModPlatform.Initialize", () =>
            {
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
            });
            yield return null;

            RunPhaseWithAbortGuard("ProfileManager.Initialize", () =>
            {
                try
                {
                    string profilesDir = System.IO.Path.Combine(
                        BepInEx.Paths.BepInExRootPath, "dinoforge-profiles");
                    _profileManager = new Profiles.ProfileManager(profilesDir, _log);
                    _log.LogInfo($"[RuntimeDriver] ProfileManager initialised at '{profilesDir}'.");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] ProfileManager initialisation failed: {ex.Message}");
                }
            });
            yield return null;

            RunPhaseWithAbortGuard("PackSettingsStore.Initialize", () =>
            {
                try
                {
                    // Fix(iter-148): use BepInEx root path so settings land under BepInEx/,
                    // not next to the game executable (AppDomain.CurrentDomain.BaseDirectory bug).
                    var store = Settings.PackSettingsStore.GetOrCreate(BepInEx.Paths.BepInExRootPath);
                    store.SetLogger(_log);
                    _log.LogInfo($"[RuntimeDriver] PackSettingsStore initialised at '{BepInEx.Paths.BepInExRootPath}'.");
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] PackSettingsStore initialisation failed: {ex.Message}");
                }
            });
            yield return null;

            RunPhaseWithAbortGuard("MainThreadDispatcher/DebugOverlay", () =>
            {
                // Add MainThreadDispatcher for IPC bridge support.
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
                // Key mapping: F9=Debug panel, F10=Mods menu (#944 fix: correct swap from ff1455b2)
                DebugLog.Write("Plugin", "[RuntimeDriver] Key mapping: F9=Debug, F10=Mods");
                Bridge.KeyInputSystem.OnF9Pressed = () =>
                {
                    try
                    {
                        DebugLog.Write("Plugin", "[RuntimeDriver] F9 pressed → DEBUG panel (via KeyInputSystem)");
                        if (_uguiReady && _dfCanvas != null)
                        {
                            _dfCanvas.ToggleDebug();
                            // ForceRefresh after toggle so the panel always shows current data
                            // (Update() never fires in DINO — periodic refresh is dead code).
                            if (_dfCanvas.DebugPanel != null && _dfCanvas.DebugPanel.IsVisible)
                            {
                                _dfCanvas.DebugPanel.ForceRefresh();
                            }
                        }
                        else _debugOverlay?.Toggle();
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("Plugin", $"[RuntimeDriver] F9 toggle failed: {ex.GetType().Name} - {ex.Message}");
                    }
                };
                Bridge.KeyInputSystem.OnF10Pressed = () =>
                {
                    try
                    {
                        DebugLog.Write("Plugin", "[RuntimeDriver] F10 pressed → MODS menu (via KeyInputSystem)");
                        if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
                        else _modMenuHost?.Toggle();
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("Plugin", $"[RuntimeDriver] F10 toggle failed: {ex.GetType().Name} - {ex.Message}");
                    }
                };

                // ── Wire HMR pack reload callback (can be invoked from background thread) ──
                Bridge.KeyInputSystem.OnPackReloadRequested = () =>
                {
                    try
                    {
                        DebugLog.Write("Plugin", "[RuntimeDriver] Pack reload requested (via OnPackReloadRequested)");
                        RequestPackReload("OnPackReloadRequested");
                    }
                    catch (Exception ex)
                    {
                        _log?.LogWarning($"[RuntimeDriver] Pack reload request failed: {ex}");
                    }
                };
            });
            yield return null;

            // ── Step 2: Attempt UGUI canvas setup ───────────────────────────────────
            // DFCanvas.Initialize() builds the canvas hierarchy synchronously and calls
            // OnInitSuccess immediately if successful, or OnInitFailed if it throws.
            // We register both callbacks so that _uguiReady is set on the main thread,
            // not from the background polling thread (which would cause UnityException).
            RunPhaseWithAbortGuard("DFCanvas.Initialize", () =>
            {
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
                        DebugLog.Write("Plugin", "[RuntimeDriver] DFCanvas.OnInitSuccess: UGUI is ready.");
                        WireUguiToModPlatform();
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
            });
            yield return null;

            RunPhaseWithAbortGuard("NativeMenuInjector/HMR/startup", () =>
            {
                // ── Step 3: Add NativeMenuInjector for main menu button injection ──────
                // This component monitors scene changes and injects a "Mods" button into
                // the native game menus (main menu, pause menu) next to Settings/Options.
                try
                {
                    _nativeMenuInjector = gameObject.AddComponent<NativeMenuInjector>();
                    _nativeMenuInjector.SetLogger(_log);
                    // Fix (iter-149): wire the pack-data provider so the native MODS page
                    // (TryShowNativeModsPage) can populate its INSTALLED PACKS list. Without
                    // this, PackDataProvider stays null → SetPacks() is never called → the
                    // left pack list renders empty even though packs are loaded.
                    _nativeMenuInjector.PackDataProvider = () =>
                        _modPlatform?.GetLoadedPackDisplayInfos()
                        ?? (System.Collections.Generic.IReadOnlyList<PackDisplayInfo>)System.Array.Empty<PackDisplayInfo>();
                    // Quick panel reads the active total_conversion ui_theme from disk.
                    _nativeMenuInjector.PacksDirectory = _modPlatform?.PacksDirectory;
                    // Route quick-panel / native-page pack toggles + reloads through the same
                    // queued path the UGUI menu uses (SetPackEnabled persists disabled_packs.json).
                    _nativeMenuInjector.OnNativePackToggled = (packId, enabled) => RequestPackToggle(packId, enabled);
                    _nativeMenuInjector.OnNativeReloadRequested = () => RequestPackReload("native mods menu reload");
                    TryWireNativeMenuInjectorHost();
                    // SPEC-002 F-07: main-thread re-scan hook for tests/tooling (not background thread — ADR-015).
                    NativeMenuInjector.OnScanNeeded = () =>
                    {
                        try { _nativeMenuInjector?.TryInjectMenuButton(); }
                        catch { /* safe-swallow: TryInjectMenuButton already logs; external trigger must not throw */ }
                    };
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
                if (Plugin._enableHotReload?.Value != false)
                {
                    // Create the tiered reloader so the watcher can classify signals.
                    // The reloader captures the loaded-DLL hash at construction time.
                    try
                    {
                        string runtimeDllPath = System.IO.Path.Combine(
                            BepInEx.Paths.PluginPath, "DINOForge.Runtime.dll");
                        _hmrTieredReloader = new HotReload.HmrTieredReloader(
                            _log,
                            packActions: new HmrPackActionsAdapter(this),
                            uiActions: new HmrUiActionsAdapter(this),
                            runtimeDllPath: runtimeDllPath);
                        _log.LogInfo("[RuntimeDriver] HmrTieredReloader created.");
                    }
                    catch (Exception ex)
                    {
                        _log.LogWarning($"[RuntimeDriver] HmrTieredReloader creation failed (will use flat reload): {ex}");
                    }

                    StartHmrWatcher();
                }
                else
                {
                    _log.LogInfo("[RuntimeDriver] HMR disabled via config (General.EnableHotReload=false).");
                }

                // ── Step 5: Start background polling (ECS world, catalog rebuild, heartbeats) ──
                // MonoBehaviour.Update() NEVER fires in DINO — background thread polling is required.
                StartBackgroundPollingThread();
            });

            // ── Step 6: Log key handler registration ────────────────────────────────
            DebugLog.Write("Plugin", $"[RuntimeDriver.Initialize] ENTRY — Initialize starting on {gameObject.name}");
            _log.LogInfo($"[RuntimeDriver] F9/F10 key handlers registered on {gameObject.name}.");

            // ── Step 6.5: Create themed loading screen (EPIC-027) ───────────────────
            // Full-screen branded loading takeover during the ~30-45s mod-init phase.
            // For an active total_conversion pack with a declared loading_screen, this
            // paints the pack's themed background + logo + tips. Hidden when the
            // MainMenu scene + engine UI are ready.
            RunPhaseWithAbortGuard("LoadingScreenController.Create", () =>
            {
                try
                {
                    // Reuse the early instance created in Plugin.Awake if it is still alive;
                    // only create a new one if it was never built or already faded out.
                    _loadingScreen = LoadingScreenController.Instance;
                    if (_loadingScreen == null)
                    {
                        string packsDir = _modPlatform?.PacksDirectory
                            ?? System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_packs");
                        _loadingScreen = LoadingScreenController.Create(gameObject, packsDir, _log);
                    }
                    if (_loadingScreen != null)
                    {
                        _loadingScreen.EnsureVisible();
                        _log.LogInfo("[RuntimeDriver] LoadingScreenController ready.");
                    }
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] LoadingScreenController creation failed: {ex}");
                }
            });
            yield return null;

            // ── Step 7: MainMenu-mode pack-load (no ECS world needed) ────────────────
            // Pack loading is YAML parsing — it does NOT require an ECS World.
            // OnWorldReadyCoroutine only fires when gameplay starts (ECS world created).
            // At main menu there is no ECS world, so packs would never load without this path.
            DebugLog.Write("Plugin", $"[RuntimeDriver] Step 7 ENTERING MainMenu-mode PackLoad — _modPlatform={((_modPlatform != null) ? "present" : "NULL")}");
            RunPhaseWithAbortGuard("MainMenu-mode PackLoad", () =>
            {
                RunMainMenuInit("initialize");

                // Subscribe to scene changes ONCE so re-entering a menu scene (e.g. returning
                // from gameplay to the main menu) re-runs the idempotent menu-mode init. This is
                // the self-healing path that recovers the engine UI after scene transitions.
                if (!_sceneChangeSubscribed)
                {
                    SceneManager.activeSceneChanged += OnRuntimeDriverSceneChanged;
                    _sceneChangeSubscribed = true;
                    _log.LogInfo("[RuntimeDriver] Subscribed activeSceneChanged for engine-UI self-heal.");
                }
            });

            // ── Step 8: Fire update check on the thread pool (best-effort, never blocks) ──
            RunPhaseWithAbortGuard("UpdateChecker.Launch", () =>
            {
                if (_modPlatform != null && !_updateCheckPushed)
                {
                    try
                    {
                        string bepInExRoot = BepInEx.Paths.BepInExRootPath;
                        string dinoForgeVersion = PluginInfo.VERSION;
                        IReadOnlyList<Updates.PackUpdateTarget> packTargets =
                            _modPlatform.GetPackUpdateTargets();
                        Updates.UpdateChecker checker = new Updates.UpdateChecker(bepInExRoot);
                        System.Threading.CancellationToken ct =
                            new System.Threading.CancellationToken(false);
                        _updateCheckTask = checker.RunAllChecksAsync(packTargets, dinoForgeVersion, ct);
                        _log.LogInfo($"[RuntimeDriver] Update check launched for DINOForge + {packTargets.Count} pack(s).");
                    }
                    catch (Exception updateEx)
                    {
                        _log.LogWarning($"[RuntimeDriver] Update check launch failed: {updateEx.Message}");
                    }
                }
            });

            if (Plugin._showOverlayOnStart?.Value == true && _dfCanvas != null)
            {
                _dfCanvas.ToggleDebug();
                _log.LogInfo("[RuntimeDriver] F9 overlay shown on start (General.ShowDebugOverlayOnStart=true).");
            }

            _log.LogInfo("[RuntimeDriver] Waiting for ECS World (Update polling)...");
            _log.LogInfo("[DINOForge] RuntimeDriver.Initialize() EXIT");

            // Pump deferred work on the main thread until destruction.
            int _themeRetryCount = 0;
            while (!_destroyed)
            {
                // ── Engine-UI self-healing bounded retry ─────────────────────────
                // Re-attempt MODS-button injection until it succeeds or the retry budget
                // is spent. The native menu canvas / custom Selectable buttons may not be
                // present on the exact frame Step 7 ran, so a single missed window would
                // otherwise leave "no Mods button" until the next scene change. Re-running
                // each pump frame on the main thread closes that race deterministically.
                if (_nativeMenuInjector != null
                    && !_nativeMenuInjector.IsModsButtonInjected
                    && _menuInitRetryFrames < MenuInitMaxRetryFrames)
                {
                    _menuInitRetryFrames++;
                    if (_menuInitRetryFrames % 30 == 0) // ~twice/sec at 60fps; cheap canvas scan
                    {
                        try { _nativeMenuInjector.TryInjectMenuButton(); }
                        catch (Exception injEx)
                        {
                            // Surface, don't swallow (Pattern #104/#111).
                            _log?.LogWarning($"[RuntimeDriver] Engine-UI retry injection failed: {injEx.Message}");
                        }
                        // Emit the heartbeat once injection succeeds (or once the budget is
                        // exhausted) so the log shows the final engine-UI state at a glance.
                        if (_nativeMenuInjector.IsModsButtonInjected
                            || _menuInitRetryFrames >= MenuInitMaxRetryFrames)
                        {
                            LogEngineUiHeartbeat("self-heal retry");
                        }
                    }
                }

                // Retry MainMenuThemer if canvas wasn't ready during Step 7
                if (_mainMenuThemer != null && !_mainMenuThemer.IsApplied && _modPlatform != null && _themeRetryCount < 30)
                {
                    _themeRetryCount++;
                    if (_themeRetryCount % 5 == 0) // every ~5 frames
                    {
                        try
                        {
                            var packInfos = _modPlatform.GetLoadedPackDisplayInfos();
                            if (packInfos.Count > 0)
                                _mainMenuThemer.TryApplyTheme(packInfos);
                        }
                        catch { /* safe-swallow: theme retry is best-effort */ }
                    }
                }

                // Re-skin non-MainMenu pages on a steady cadence. Settings sub-tabs and the
                // game create/select screens are instantiated lazily when the user navigates,
                // so a one-shot apply misses them. The reskinner is idempotent (per-object
                // marker) — repeated passes only touch newly-appeared elements.
                if (_canvasReskinner != null && _modPlatform != null)
                {
                    _reskinRetryCount++;
                    if (_reskinRetryCount % 15 == 0) // every ~15 frames
                    {
                        try
                        {
                            _canvasReskinner.ReskinAllPages(_modPlatform.GetLoadedPackDisplayInfos());
                        }
                        catch { /* safe-swallow: page reskin retry is best-effort */ }
                    }
                }

                // ── Step 8 deferred: push update-check results to UI once the Task completes ──
                if (!_updateCheckPushed && _updateCheckTask != null
                    && _updateCheckTask.IsCompleted && _dfCanvas?.ModMenuPanel != null)
                {
                    _updateCheckPushed = true;
                    try
                    {
                        System.Collections.Generic.IReadOnlyList<Updates.UpdateInfo> updates =
                            _updateCheckTask.Result;
                        if (updates.Count > 0)
                        {
                            _dfCanvas.ModMenuPanel.SetUpdatesAvailable(updates);
                            _log?.LogInfo($"[RuntimeDriver] Update check: {updates.Count} update(s) pushed to UI.");
                        }
                        else
                        {
                            _log?.LogInfo("[RuntimeDriver] Update check: up to date.");
                        }
                    }
                    catch (Exception updateEx)
                    {
                        // safe-swallow: update-check result delivery is best-effort
                        _log?.LogWarning($"[RuntimeDriver] Update check result delivery failed: {updateEx.Message}");
                    }
                    _updateCheckTask = null;
                }

                if (TryDequeuePendingWorldReady(out World? pendingWorld))
                {
                    yield return ProcessWorldReadyCoroutine(pendingWorld!);
                    continue;
                }

                if (TryDequeuePendingPackReload(out string? packReloadReason))
                {
                    yield return ProcessPackReloadCoroutine(packReloadReason!);
                    continue;
                }

                if (TryDequeuePendingPackToggle(out string? packId, out bool enabled))
                {
                    yield return ProcessPackToggleCoroutine(packId!, enabled);
                    continue;
                }

                // Deferred catalog rebuild (queued from background thread to avoid EntityManager race)
                World? catalogWorld = null;
                lock (_deferredWorkLock)
                {
                    if (_pendingCatalogWorld != null)
                    {
                        catalogWorld = _pendingCatalogWorld;
                        _pendingCatalogWorld = null;
                    }
                }
                if (catalogWorld != null && catalogWorld.IsCreated)
                {
                    try
                    {
                        _log?.LogInfo($"[RuntimeDriver] Catalog rebuild executing on main thread for world '{catalogWorld.Name}'");
                        _modPlatform?.RebuildCatalogAndApplyStats(catalogWorld);
                    }
                    catch (Exception ex)
                    {
                        _log?.LogWarning($"[RuntimeDriver] Catalog rebuild failed: {ex.Message}");
                    }
                }

                yield return null;
            }
        }

        private void RunPhaseWithAbortGuard(string phaseName, Action phase)
        {
            try
            {
                phase();
            }
            catch (ThreadAbortException)
            {
                try
                {
#pragma warning disable SYSLIB0006 // Required here to clear Unity's abort and preserve the rest of the teardown path.
                    Thread.ResetAbort();
#pragma warning restore SYSLIB0006
                }
                catch (Exception resetEx)
                {
                    _log?.LogWarning($"[RuntimeDriver] {phaseName} abort reset failed: {resetEx}");
                }

                _log?.LogWarning($"[RuntimeDriver] {phaseName} aborted by Unity thread abort.");
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[RuntimeDriver] {phaseName} failed: {ex}");
            }
        }

        // ------------------------------------------------------------------ //
        // Engine-UI MainMenu-mode init (deterministic, idempotent, self-healing)
        // ------------------------------------------------------------------ //

        /// <summary>
        /// Loads packs, wires the UGUI mod menu, and attempts native MODS-button injection
        /// WITHOUT requiring an ECS World. DINO only creates ECS worlds when entering gameplay,
        /// so the ECS-gated <see cref="ProcessWorldReadyCoroutine"/> never runs at the main menu —
        /// this is the only path that brings up the engine UI there.
        ///
        /// Idempotent: safe to call repeatedly. <see cref="ModPlatform.LoadPacks"/> is pure YAML
        /// parsing, and <see cref="UI.NativeMenuInjector.TryInjectMenuButton"/> short-circuits when
        /// the MODS button already exists. Every failure is logged (no silent swallow — Pattern
        /// #104/#111) so the cause is visible in the BepInEx console.
        /// </summary>
        /// <param name="reason">Diagnostic tag for the log (e.g. "initialize", "scene-change").</param>
        private void RunMainMenuInit(string reason)
        {
            if (_modPlatform == null)
            {
                _log.LogWarning($"[RuntimeDriver] MainMenu-mode init ({reason}) skipped — _modPlatform is null.");
                return;
            }

            try
            {
                _log.LogInfo($"[RuntimeDriver] MainMenu-mode init ({reason}): calling LoadPacks() (no ECS world required).");
                ContentLoadResult result = _modPlatform.LoadPacks();
                _log.LogInfo($"[RuntimeDriver] MainMenu-mode init ({reason}) pack-load complete: success={result.IsSuccess}, loaded={result.LoadedPacks.Count}, errors={result.Errors.Count}");

                WireUguiToModPlatform();
                PushLoadedPacksToUgui("main-menu init");

                // Hide loading screen now that packs are loaded.
                if (_loadingScreen != null)
                {
                    _loadingScreen.BeginFadeOut();
                    _log.LogInfo("[RuntimeDriver] LoadingScreenController faded out (MainMenu-mode init complete).");
                }

                // Apply total_conversion theme to main menu (best-effort; pump loop retries).
                try
                {
                    _mainMenuThemer = new MainMenuThemer(_log, _modPlatform.PacksDirectory);
                    IReadOnlyList<PackDisplayInfo> packInfos = _modPlatform.GetLoadedPackDisplayInfos();
                    _mainMenuThemer.TryApplyTheme(packInfos);

                    // Color-skin every non-MainMenu page (Settings + GAME/VIDEO/SOUND/CONTROLS/
                    // TWITCH sub-tabs, game create/select) with the active total_conversion theme.
                    // Sub-panels are created lazily on navigation, so the pump loop re-runs this.
                    _canvasReskinner = new UI.CanvasReskinner(_log, _modPlatform.PacksDirectory);
                    _canvasReskinner.Invalidate();
                    _reskinRetryCount = 0;
                    _canvasReskinner.ReskinAllPages(packInfos);
                }
                catch (Exception themeEx)
                {
                    _log.LogWarning($"[RuntimeDriver] MainMenuThemer failed: {themeEx.Message}");
                }

                // Kick a native injection attempt immediately; the pump loop bounded-retry
                // handles the case where the menu canvas is not ready on this exact frame.
                if (_nativeMenuInjector != null)
                {
                    try { _nativeMenuInjector.TryInjectMenuButton(); }
                    catch (Exception injEx)
                    {
                        _log.LogWarning($"[RuntimeDriver] MainMenu-mode init ({reason}) injection attempt failed: {injEx.Message}");
                    }
                }

                // Emit the single launch-time engine-UI heartbeat (idempotent: only once unless
                // a scene change re-arms it). If injection is still pending the pump-loop retry
                // re-emits with the final state.
                LogEngineUiHeartbeat(reason);
            }
            catch (Exception ex)
            {
                _log.LogError($"[RuntimeDriver] MainMenu-mode init ({reason}) FAILED: {ex}");
            }
        }

        /// <summary>
        /// Self-heal hook: when DINO transitions to a menu scene (no active gameplay ECS world),
        /// re-arm the bounded retry and re-run the idempotent menu-mode init so the engine UI is
        /// rebuilt after returning from gameplay. Never throws to Unity.
        /// </summary>
        private void OnRuntimeDriverSceneChanged(Scene previous, Scene next)
        {
            try
            {
                if (_destroyed) return;
                _log.LogInfo($"[RuntimeDriver] activeSceneChanged: '{previous.name}' → '{next.name}' — re-arming engine-UI menu-mode init.");
                // Re-arm: the scene swap destroyed the previous canvas + injected button, so allow
                // a fresh injection attempt and a fresh heartbeat for the new scene.
                _menuInitRetryFrames = 0;
                _engineUiHeartbeatLogged = false;
                RunMainMenuInit("scene-change");
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[RuntimeDriver] OnRuntimeDriverSceneChanged failed: {ex.Message}");
            }
        }

        /// <summary>
        /// Emits a single unambiguous launch-time heartbeat summarising engine-UI readiness so the
        /// user (and tooling) can confirm state at a glance from the BepInEx console / LogOutput.log.
        /// Logged at most once per scene (re-armed on scene change).
        /// </summary>
        private void LogEngineUiHeartbeat(string reason)
        {
            if (_engineUiHeartbeatLogged) return;

            int packs = 0;
            try { packs = _modPlatform?.GetLoadedPackDisplayInfos().Count ?? 0; }
            catch { /* safe-swallow: heartbeat is diagnostic-only and must not throw */ }

            bool modsButton = _nativeMenuInjector != null && _nativeMenuInjector.IsModsButtonInjected;
            bool f9 = _debugOverlay != null || _dfCanvas != null;       // F9 debug panel host present
            bool f10 = _modMenuHost != null || _dfCanvas?.ModMenuPanel != null; // F10 mods panel host present

            // Only mark the heartbeat as "logged" (final) once the MODS button is in OR we were
            // called from the retry path; the first injectionless call may re-emit after retries.
            if (modsButton || string.Equals(reason, "self-heal retry", StringComparison.Ordinal))
            {
                _engineUiHeartbeatLogged = true;
            }

            string readyLine = $"[DINOForge] ENGINE-UI READY: packs={packs} modsButton={modsButton} f9={f9} f10={f10} (via {reason})";
            _log.LogInfo(readyLine);
            // iter-149b: also mirror to dinoforge_debug.log so live verification (which reads the
            // DINOForge debug log, not BepInEx LogOutput.log) can confirm engine-UI readiness.
            DebugLog.Write("Plugin", readyLine);
        }

        internal void RequestPackReload(string reason)
        {
            lock (_deferredWorkLock)
            {
                _pendingPackReload = true;
                _pendingPackReloadReason = reason;
            }
        }

        private void RequestPackToggle(string packId, bool enabled)
        {
            lock (_deferredWorkLock)
            {
                _pendingPackToggle = true;
                _pendingPackToggleId = packId;
                _pendingPackToggleEnabled = enabled;
            }
        }

        private bool TryDequeuePendingWorldReady(out World? world)
        {
            lock (_deferredWorkLock)
            {
                if (_hasPendingWorldReady && !_worldReadyProcessing && _pendingWorldReady != null)
                {
                    world = _pendingWorldReady;
                    _pendingWorldReady = null;
                    _hasPendingWorldReady = false;
                    _worldReadyProcessing = true;
                    return true;
                }
            }

            world = null;
            return false;
        }

        private bool TryDequeuePendingPackReload(out string? reason)
        {
            lock (_deferredWorkLock)
            {
                if (_pendingPackReload && !_worldReadyProcessing && !_pendingPackToggle)
                {
                    reason = _pendingPackReloadReason ?? "queued";
                    _pendingPackReload = false;
                    _pendingPackReloadReason = null;
                    return true;
                }
            }

            reason = null;
            return false;
        }

        private bool TryDequeuePendingPackToggle(out string? packId, out bool enabled)
        {
            lock (_deferredWorkLock)
            {
                if (_pendingPackToggle && !_worldReadyProcessing)
                {
                    packId = _pendingPackToggleId;
                    enabled = _pendingPackToggleEnabled;
                    _pendingPackToggle = false;
                    _pendingPackToggleId = null;
                    return !string.IsNullOrEmpty(packId);
                }
            }

            packId = null;
            enabled = false;
            return false;
        }

        private IEnumerator ProcessWorldReadyCoroutine(World ecsWorld)
        {
            try
            {
                _log.LogInfo($"[RuntimeDriver] ECS World available: {ecsWorld.Name}");
                _registeredWorldInstance = ecsWorld;

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

                if (_modPlatform == null)
                {
                    yield break;
                }

                yield return null;

                RunPhaseWithAbortGuard("ModPlatform.OnWorldReady", () =>
                {
                    _modPlatform.OnWorldReady(ecsWorld);
                    _log.LogInfo("[RuntimeDriver] ModPlatform notified of world readiness.");
                });

                WireUguiToModPlatform();

                yield return null;

                ContentLoadResult loadResult = null!;
                bool loadCompleted = false;
                ModPlatform modPlatform = _modPlatform;
                RunPhaseWithAbortGuard("ModPlatform.LoadPacks", () =>
                {
                    loadResult = modPlatform.LoadPacks();
                    loadCompleted = true;
                });

                if (loadCompleted)
                {
                    _log?.LogInfo($"[RuntimeDriver.diag] LoadPacks returned, modPlatformReady={modPlatform != null}, packCount={loadResult.LoadedPacks.Count} — entering UGUI push block");
                    _log?.LogInfo($"[RuntimeDriver] Pack loading complete: success={loadResult.IsSuccess}, " +
                        $"loaded={loadResult.LoadedPacks.Count}, errors={loadResult.Errors.Count}");
                    _log?.LogInfo($"[RuntimeDriver.diag] ABOUT TO CALL PushLoadedPacksToUgui('initial load') — dfCanvas={_dfCanvas != null}, modPlatform={modPlatform != null}");
                    PushLoadedPacksToUgui("initial load");

                    // Hide the loading screen now that world is ready and packs are loaded
                    if (_loadingScreen != null)
                    {
                        _loadingScreen.BeginFadeOut();
                        _log?.LogInfo("[RuntimeDriver] LoadingScreenController faded out (world ready).");
                    }
                }

                yield return null;

                RunPhaseWithAbortGuard("ModPlatform.StartHotReload", () =>
                {
                    modPlatform?.StartHotReload();
                    _log?.LogInfo("[RuntimeDriver] Hot reload started.");
                });

                yield return null;

                RunPhaseWithAbortGuard("ModSettingsPanel.DiscoverSettings", () =>
                {
                    if (_modSettingsHost is ModSettingsPanel settingsPanel)
                    {
                        settingsPanel.DiscoverSettings();
                        _log?.LogInfo("[RuntimeDriver] Mod settings discovered.");
                    }
                });

                if (_debugOverlay != null)
                {
                    _debugOverlay.SetModPlatform(modPlatform);
                }
            }
            finally
            {
                lock (_deferredWorkLock)
                {
                    _worldReadyProcessing = false;
                }
            }
        }

        private IEnumerator ProcessPackReloadCoroutine(string reason)
        {
            if (_modPlatform == null)
            {
                yield break;
            }

            _log.LogInfo($"[RuntimeDriver] Processing deferred pack reload ({reason}).");
            yield return null;

            ContentLoadResult loadResult = null!;
            bool loadCompleted = false;
            RunPhaseWithAbortGuard("ModPlatform.LoadPacks", () =>
            {
                loadResult = _modPlatform.LoadPacks();
                loadCompleted = true;
            });

            if (loadCompleted)
            {
                _log.LogInfo($"[RuntimeDriver] Deferred pack reload complete: success={loadResult.IsSuccess}, " +
                    $"loaded={loadResult.LoadedPacks.Count}, errors={loadResult.Errors.Count}");
                _log?.LogInfo($"[RuntimeDriver.diag] ABOUT TO CALL PushLoadedPacksToUgui('deferred reload') — dfCanvas={_dfCanvas != null}, modPlatform={_modPlatform != null}");
                PushLoadedPacksToUgui("deferred reload");

                // Update header status line and show toast so the user knows reload completed.
                string statusMsg = loadResult.IsSuccess
                    ? $"Reloaded — {loadResult.LoadedPacks.Count} pack(s) loaded"
                    : $"Reload failed — {loadResult.Errors.Count} error(s)";
                _dfCanvas?.ModMenuPanel?.SetStatus(statusMsg, loadResult.Errors.Count);
                ToastType toastType = loadResult.IsSuccess ? ToastType.Info : ToastType.Warning;
                _dfCanvas?.ShowToast(statusMsg, toastType);
            }

            yield return null;
        }

        private IEnumerator ProcessPackToggleCoroutine(string packId, bool enabled)
        {
            if (_modPlatform == null)
            {
                yield break;
            }

            _log.LogInfo($"[RuntimeDriver] Processing deferred pack toggle: {packId} enabled={enabled}.");
            yield return null;

            RunPhaseWithAbortGuard("ModPlatform.SetPackEnabled", () =>
            {
                _modPlatform.SetPackEnabled(packId, enabled);
            });

            yield return ProcessPackReloadCoroutine($"pack toggle {packId}");

            if (_dfCanvas?.ModMenuPanel != null)
            {
                _dfCanvas.ModMenuPanel.SetStatus($"Pack '{packId}' {(enabled ? "enabled" : "disabled")} and reloaded");
            }
        }

        private void PushLoadedPacksToUgui(string reason)
        {
            _log?.LogInfo($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) ENTRY: dfCanvas={(_dfCanvas != null ? "OK" : "NULL")}, modPlatform={(_modPlatform != null ? "OK" : "NULL")}, modMenuPanel={(_dfCanvas?.ModMenuPanel != null ? "OK" : "NULL")}, hasLastLoadResult={_modPlatform?.HasLastLoadResult.ToString() ?? "NULL"}, lastLoad={_modPlatform?.DescribeLastLoadResult() ?? "modPlatform=NULL"}");

            if (_dfCanvas == null)
            {
                _log?.LogWarning($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) skipped — _dfCanvas is NULL.");
                return;
            }

            if (_modPlatform == null)
            {
                _log?.LogWarning($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) skipped — _modPlatform is NULL.");
                return;
            }

            if (_dfCanvas.ModMenuPanel == null)
            {
                _log?.LogWarning($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) skipped — ModMenuPanel is NULL.");
                return;
            }

            try
            {
                IReadOnlyList<PackDisplayInfo> packInfos = _modPlatform.GetLoadedPackDisplayInfos();
                _log?.LogInfo($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) resolved packInfos.Count={packInfos.Count}; {_modPlatform.DescribeLastLoadResult()}");
                if (packInfos.Count == 0)
                {
                    _log?.LogWarning($"[RuntimeDriver] PushLoadedPacksToUgui({reason}) resolved 0 packs — registry or load-result path may be empty.");
                }

                _dfCanvas.ModMenuPanel.SetPacks(packInfos);

                ContentLoadResult? lastResult = _modPlatform.GetLastLoadResult();
                if (lastResult != null)
                {
                    int errorCount = lastResult.Errors.Count;
                    string statusMsg = lastResult.IsSuccess
                        ? $"{lastResult.LoadedPacks.Count} packs loaded"
                        : $"{lastResult.LoadedPacks.Count} loaded, {errorCount} error(s)";
                    _dfCanvas.ModMenuPanel.SetStatus(statusMsg, errorCount);
                }

                _log?.LogInfo($"[RuntimeDriver] UGUI mod menu refreshed after {reason}.");
            }
            catch (Exception ex)
            {
                _log?.LogWarning($"[RuntimeDriver] Failed to refresh UGUI mod menu after {reason}: {ex}");
            }
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
                try { System.Threading.Thread.CurrentThread.Name = "DINOForge.HmrWatcher"; } catch { /* safe-swallow: thread name set is best-effort diagnostics */ }
                try
                {
                    string signalPath = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "DINOForge_HotReload");
                    // #873: cooperative shutdown — observe _destroyed AND wake immediately via stop-event
                    // mirrors StartBackgroundPollingThread (line 920+) pattern.
                    while (!_destroyed)
                    {
                        // Wait returns true when the event is signaled (OnDestroy) → exit promptly.
#pragma warning disable DF0116 // Intentional blocking poll interval on the background watcher thread.
                        if (_backgroundPollStopEvent.Wait(2000))
#pragma warning restore DF0116
                        {
                            break;
                        }
                        if (_destroyed) break;

                        if (System.IO.File.Exists(signalPath))
                        {
                            // Read optional path hint written alongside the signal (first line of file).
                            string changedPath = string.Empty;
                            try
                            {
                                string signalContent = System.IO.File.ReadAllText(signalPath).Trim();
                                changedPath = signalContent;
                            }
                            catch { } // safe-swallow: path hint is optional; empty string → HandleUnknown()

                            try { System.IO.File.Delete(signalPath); } catch { } // safe-swallow: HMR signal file cleanup, non-critical

                            _log?.LogInfo($"[RuntimeDriver] HMR: Signal detected. changedPath='{changedPath}'");

                            // #898: tiered reload — classify changed path and act accordingly.
                            HotReload.HmrTieredReloader? reloader = _hmrTieredReloader;
                            if (reloader != null)
                            {
                                try
                                {
                                    if (string.IsNullOrEmpty(changedPath))
                                        reloader.HandleUnknown();
                                    else
                                        reloader.Handle(changedPath);
                                    _log?.LogInfo("[RuntimeDriver] HMR: Tiered reloader handled signal.");
                                }
                                catch (System.Exception ex)
                                {
                                    _log?.LogWarning($"[RuntimeDriver] HMR: TieredReloader.Handle failed, falling back to flat reload: {ex}");
                                    // Fall through to legacy path below
                                    reloader = null;
                                }
                            }

                            if (reloader == null)
                            {
                                // #891: legacy flat reload path — used when tiered reloader is unavailable.
                                try
                                {
                                    RuntimeDriver? driver = Plugin.PersistentRoot?.GetComponent<RuntimeDriver>();
                                    if (driver != null)
                                    {
                                        driver.RequestPackReload("HMR signal (fallback)");
                                    }
                                    else
                                    {
                                        Bridge.KeyInputSystem.OnPackReloadRequested?.Invoke();
                                    }
                                }
                                catch (System.Exception ex)
                                {
                                    _log?.LogWarning($"[RuntimeDriver] HMR: Pack reload enqueue failed: {ex}");
                                }
                            }

                            _log?.LogInfo("[RuntimeDriver] HMR: Signal handling complete.");
                        }
                    }
                    // #873: explicit exit log — proves thread terminated cleanly on OnDestroy.
                    _log?.LogInfo("[RuntimeDriver] HMR watcher thread exiting (destroyed=true)");
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
                try { System.Threading.Thread.CurrentThread.Name = "DINOForge.BackgroundPoll"; } catch { /* safe-swallow: thread name set is best-effort diagnostics */ }
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
                                DebugLog.Write("Plugin", "[RuntimeDriver] Background poll: calling TryResurrect (deferred)");
                                Plugin.TryResurrect(Plugin.LastSceneNameForResurrection ?? "unknown", "BackgroundPoll_Deferred");
                            }
                            catch (Exception ex)
                            {
                                DebugLog.Write("Plugin", $"[RuntimeDriver] Deferred TryResurrect failed: {ex.Message}");
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
#pragma warning disable CS0162 // Disabled reference block kept for operator comparison during hotfix validation.
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
#pragma warning restore CS0162

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
                        // World found — detect world CHANGES and re-trigger OnWorldReady
                        else
                        {
                            if (_destroyed) break;
                            try
                            {
                                World? w = World.DefaultGameObjectInjectionWorld;
                                if (w != null && w.IsCreated)
                                {
                                    // Detect world change: new world created after scene transition
                                    if (!ReferenceEquals(_registeredWorldInstance, w))
                                    {
                                        _log?.LogInfo($"[RuntimeDriver] World changed: '{w.Name}' (was {(_registeredWorldInstance != null ? _registeredWorldInstance.Name : "null")})");
                                        TryRegisterKeyInputSystem(w);

                                        // Re-trigger OnWorldReady for the new world
                                        _worldFound = false;
                                        _catalogRebuilt = false;
                                        _worldFound = true;
                                        DebugLog.Write("Plugin", $"[RuntimeDriver] World change detected — queueing OnWorldReady for '{w.Name}'");
                                        OnWorldReady(w);
                                    }

                                    // Deferred catalog rebuild: queue to main thread, don't call from BG thread
                                    if (!_catalogRebuilt)
                                    {
                                        int entityCount = w.EntityManager.UniversalQuery.CalculateEntityCount();
                                        if (entityCount > 1000)
                                        {
                                            _catalogRebuilt = true;
                                            _log?.LogInfo($"[RuntimeDriver] Catalog rebuild deferred to main thread ({entityCount} entities)");
                                            lock (_deferredWorkLock)
                                            {
                                                _pendingCatalogWorld = w;
                                            }
                                        }
                                    }
                                }
                                else if (w == null || !w.IsCreated)
                                {
                                    // World was destroyed (scene transition) — reset for next world
                                    if (_worldFound)
                                    {
                                        _worldFound = false;
                                        _catalogRebuilt = false;
                                        DebugLog.Write("Plugin", "[RuntimeDriver] World destroyed — reset worldFound, will re-detect");
                                    }
                                }
                            }
                            catch { } // safe-swallow: ECS world discovery best-effort
                        }
                        // (World-change detection + KeyInputSystem re-registration merged into the else block above)
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
            if (ReferenceEquals(_modMenuHost, _dfCanvas.ModMenuPanel))
            {
                TryWireNativeMenuInjectorHost();
                return;
            }
            ModPlatform platform = _modPlatform;

            try
            {
                if (_dfCanvas.ModMenuPanel != null)
                {
                    _dfCanvas.ModMenuPanel.OnReloadRequested = () => RequestPackReload("UGUI reload button");
                }

                IModSettingsHost settingsHost = new NoOpSettingsHost();

                if (_dfCanvas.ModMenuPanel == null)
                {
                    throw new InvalidOperationException("DFCanvas did not create ModMenuPanel.");
                }

                platform.SetUI(_dfCanvas.ModMenuPanel, settingsHost);
                _dfCanvas.ModMenuPanel.OnReloadRequested = () => RequestPackReload("UGUI reload button");
                _dfCanvas.ModMenuPanel.OnPackToggled = RequestPackToggle;

                // ── Profiles (#918) ──────────────────────────────────────────
                if (_profileManager != null)
                {
                    RuntimeDriver capturedDriver = this;
                    _dfCanvas.ModMenuPanel.SetProfileManager(_profileManager);
                    _dfCanvas.ModMenuPanel.OnProfileLoaded = enabledPackIds =>
                    {
                        try
                        {
                            // Disable all packs then enable only those in the profile
                            foreach (UI.PackDisplayInfo p in platform.GetLoadedPackDisplayInfos())
                            {
                                bool shouldEnable = false;
                                foreach (string id in enabledPackIds)
                                {
                                    if (string.Equals(id, p.Id, StringComparison.Ordinal))
                                    {
                                        shouldEnable = true;
                                        break;
                                    }
                                }
                                if (p.IsEnabled != shouldEnable)
                                    capturedDriver.RequestPackToggle(p.Id, shouldEnable);
                            }
                        }
                        catch (Exception ex)
                        {
                            _log?.LogWarning($"[RuntimeDriver] OnProfileLoaded failed: {ex.Message}");
                        }
                    };
                    _log.LogInfo("[RuntimeDriver] ProfileManager wired to ModMenuPanel.");
                }

                TryWireNativeMenuInjectorHost();

                // Wire UGUI DebugPanel to ModPlatform so it displays platform status
                if (_dfCanvas.DebugPanel != null && _modPlatform != null)
                {
                    _dfCanvas.DebugPanel.SetModPlatform(platform);
                    _log.LogInfo("[RuntimeDriver] UGUI DebugPanel wired to ModPlatform.");
                }

                _modMenuHost = _dfCanvas.ModMenuPanel;
                _modSettingsHost = settingsHost;

                // Wire HudStrip so it receives pack counts on every load/reload.
                if (_dfCanvas.HudStrip != null)
                {
                    UI.HudStrip hudStrip = _dfCanvas.HudStrip;
                    platform.OnHudCountsChanged = (p, e) => hudStrip.SetStatus(p, e);
                }

                _log.LogInfo("[RuntimeDriver] UGUI wired to ModPlatform via IModMenuHost.");

                _log?.LogInfo($"[RuntimeDriver.diag] ABOUT TO CALL PushLoadedPacksToUgui('late UGUI wiring') — dfCanvas={_dfCanvas != null}, modPlatform={_modPlatform != null}");
                PushLoadedPacksToUgui("late UGUI wiring immediate sync");

                // Fix #31/#32: LoadPacks() may have run before the UI host was wired
                // (ModPlatform.UpdateUI() returns early when _modMenuHost is null).
                // Now that the host is registered, replay a LoadPacks() so ModMenuPanel
                // receives the pack list and DebugPanel receives ModPlatform data.
                // This is a no-op if packs have not been loaded yet.
                if (platform.GetLoadedPackIds() != null)
                {
                    _log?.LogInfo("[RuntimeDriver] Queuing LoadPacks() to populate UGUI panels after late wiring.");
                    RequestPackReload("late UGUI wiring");
                }
            }
            catch (Exception ex)
            {
                _log.LogWarning($"[RuntimeDriver] UGUI→ModPlatform wiring failed, activating IMGUI fallback: {ex}");
                _uguiReady = false;
                ActivateImguiFallback();
            }
        }

        private void TryWireNativeMenuInjectorHost()
        {
            if (_nativeMenuInjector == null || _dfCanvas?.ModMenuPanel == null)
            {
                return;
            }

            // Fix #30/#884: UGUI can be wired before NativeMenuInjector is created. Keep this
            // idempotent and call it from both paths so the native MODS button never fires with
            // _menuHost == null.
            NativeMainMenuModMenu nativeHost = new NativeMainMenuModMenu();
            if (_log != null) nativeHost.SetLogger(_log);
            // Fix (iter-149): give the native MODS screen a live pack source. ModPlatform.UpdateUI
            // only pushes packs to the overlay host it owns, not to this contextual host, so the
            // native page would otherwise list zero packs.
            nativeHost.PackDataProvider = () =>
                _modPlatform?.GetLoadedPackDisplayInfos()
                ?? (System.Collections.Generic.IReadOnlyList<PackDisplayInfo>)System.Array.Empty<PackDisplayInfo>();
            ContextualModMenuHost contextualHost = new ContextualModMenuHost(
                _dfCanvas.ModMenuPanel, nativeHost);
            _nativeMenuInjector.SetModMenuHost(contextualHost);
            _log?.LogInfo("[RuntimeDriver] NativeMenuInjector wired via ContextualModMenuHost (native menu active, overlay fallback).");
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
            lock (_deferredWorkLock)
            {
                _pendingWorldReady = ecsWorld;
                _hasPendingWorldReady = true;
            }
        }


        private void OnDestroy()
        {
            // Iter-144 #543 fix: set resurrection flags AT THE VERY TOP, before any teardown work.
            // The s_rootJustDestroyed companion flag is the source of truth for "RuntimeDriver died
            // and has not been replaced yet" — the fallback loop checks it via OR with
            // NeedsResurrection to avoid the Unity fake-null trap where PersistentRoot reports
            // == null via operator overload but ReferenceEquals(_, null) is false.
            // s_skipBundleUnload is checked by AssetSwapSystem.OnDestroy to preserve bundles
            // across scene transitions (otherwise chicken-sprite swaps orphan mid-flight).
            Plugin.NeedsResurrection = true;
            Plugin.NeedsDeferredResurrection = true;
            Plugin.s_rootJustDestroyed = true;
            Plugin.s_skipBundleUnload = true;

            // Iter-144 #543 gray-freeze fix: signal all subsystems IMMEDIATELY, before any other
            // teardown work runs. VanillaCatalog.Build + ContentLoader pack registration check
            // this static flag and short-circuit cleanly to avoid racing world teardown.
            s_isBeingDestroyed = true;
            _destroyed = true; // Signal background polling thread to stop
            _backgroundPollStopEvent.Set();  // Wake up the polling loop

            // Pair the activeSceneChanged subscription added in Step 7 (Pattern #105). This
            // instance is being destroyed on the scene transition; the next RuntimeDriver
            // resubscribes its own handler during its Initialize Step 7.
            if (_sceneChangeSubscribed)
            {
                try { SceneManager.activeSceneChanged -= OnRuntimeDriverSceneChanged; }
                catch { /* safe-swallow: unsubscribe is best-effort during teardown */ }
                _sceneChangeSubscribed = false;
            }
            // iter-145 #882 ROOT CAUSE: Removed _resurrectionFallbackStopEvent.Set() — that was killing
            // the fallback thread on every RuntimeDriver.OnDestroy (scene transition), preventing the
            // post-OnDestroy resurrection that's the whole point of the fallback. The "wake without
            // exit" intent at L433 used `if (Wait(...)) break;` which exits unconditionally on signal
            // regardless of _resurrectionFallbackStop. Fallback thread now only exits when Plugin
            // itself unloads (via _resurrectionFallbackStop=true set elsewhere). 500ms poll latency
            // for resurrection is fine; was never real-time-critical.

            // Iter-144 #547 gray-freeze ROOT CAUSE fix: WinDbg analysis revealed the main thread
            // was parked in mono_jit_cleanup → mono_threads_set_shutting_down waiting on the
            // bridge thread stuck in synchronous ConnectNamedPipe. Force-cancel the bridge accept
            // loop NOW (synchronously) before any other teardown work, so the kernel I/O unblocks
            // and mono_jit_cleanup can complete cleanly at process exit. The bridge's
            // RequestShutdown() disposes the current pipe handle, which yields ObjectDisposedException
            // on the BeginWaitForConnection IAsyncResult and lets the accept loop exit.
            // (docs/sessions/iter144-windbg-wedge-stack.md)
            try
            {
                Plugin.SharedBridgeServer?.RequestShutdown();
                DebugLog.Write("Plugin", "[RuntimeDriver] OnDestroy: GameBridgeServer.RequestShutdown() invoked (sync pipe unwedge).");
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[RuntimeDriver] OnDestroy: RequestShutdown failed (non-fatal): {ex.GetType().Name}: {ex.Message}");
            }

            // Iter-144 #547 H5: belt-and-suspenders — the resurrection flags were already set above,
            // but null the field reference explicitly so the next check sees a true managed null
            // (not a Unity fake-null) on subsequent activeSceneChanged callbacks.
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
            DebugLog.Write("Plugin",
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
                    DebugLog.Write("Plugin", "[RuntimeDriver] OnDestroy: dispatching ShutdownNonBridge to worker thread.");
                    Thread shutdownWorker = new Thread(() =>
                    {
                        try
                        {
                            mp.ShutdownNonBridge();
                            DebugLog.Write("Plugin", "[RuntimeDriver] OnDestroy.worker: ShutdownNonBridge completed.");
                        }
                        catch (Exception ex)
                        {
                            DebugLog.Write("Plugin", $"[RuntimeDriver] OnDestroy.worker: ShutdownNonBridge threw {ex.GetType().Name}: {ex.Message}");
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
                DebugLog.Write("Plugin", $"[RuntimeDriver] OnDestroy: ShutdownNonBridge dispatch failed: {ex.Message}");
            }
            // #923: Persist metrics snapshot on shutdown (best-effort).
            try
            {
                string snapshotPath = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge-metrics-snapshot.json");
                string metricsJson = MetricsCollector.Instance.DumpJson();
                File.WriteAllText(snapshotPath, metricsJson, System.Text.Encoding.UTF8);
                DebugLog.Write("Plugin", $"[RuntimeDriver] OnDestroy: metrics snapshot written to '{snapshotPath}'.");
            }
            catch (Exception ex)
            {
                // Best-effort: metrics persistence must never throw from OnDestroy
                DebugLog.Write("Plugin", $"[RuntimeDriver] OnDestroy: metrics snapshot failed (non-fatal): {ex.Message}");
            }

            DebugLog.Write("Plugin", "[RuntimeDriver] OnDestroy: returning to Unity (resurrection flags set, fallback thread will revive).");
        }
    }

    // ── HMR adapter implementations ───────────────────────────────────────────────

    /// <summary>
    /// Bridges <see cref="HotReload.IHmrPackActions"/> to the <see cref="RuntimeDriver"/>
    /// deferred-work queue so tier-1 and tier-2 actions run safely on the Unity main thread.
    /// </summary>
    internal sealed class HmrPackActionsAdapter : HotReload.IHmrPackActions
    {
        private readonly RuntimeDriver _driver;

        internal HmrPackActionsAdapter(RuntimeDriver driver)
        {
            _driver = driver;
        }

        /// <inheritdoc/>
        public void TriggerPackReload()
        {
            // Enqueue through the existing deferred-work mechanism so LoadPacks +
            // UGUI refresh + SetStatus + ShowToast all fire from the main-thread coroutine.
            _driver.RequestPackReload("HMR tier-1");
        }

        /// <inheritdoc/>
        public void TriggerSceneReload()
        {
            // Load scene 1 (MainMenu) — asset bundles are re-evaluated on re-enter.
            try
            {
                UnityEngine.SceneManagement.SceneManager.LoadScene(1);
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[HmrPackActionsAdapter] TriggerSceneReload LoadScene(1) failed: {ex.Message}");
            }
        }
    }

    /// <summary>
    /// Bridges <see cref="HotReload.IHmrUiActions"/> to <see cref="DFCanvas"/> /
    /// <see cref="UI.ModMenuPanel"/>. Called from the HMR background thread;
    /// MonoBehaviour calls are permitted for DontDestroyOnLoad objects in Mono 2021.3
    /// (confirmed by existing F9/F10 background-thread pattern).
    /// </summary>
    internal sealed class HmrUiActionsAdapter : HotReload.IHmrUiActions
    {
        private readonly RuntimeDriver _driver;

        internal HmrUiActionsAdapter(RuntimeDriver driver)
        {
            _driver = driver;
        }

        /// <inheritdoc/>
        public void ShowToast(string message, HotReload.HmrToastKind kind)
        {
            try
            {
                UI.ToastType toastType = kind switch
                {
                    HotReload.HmrToastKind.Warning => UI.ToastType.Warning,
                    HotReload.HmrToastKind.Error => UI.ToastType.Error,
                    _ => UI.ToastType.Info,
                };

                if (_driver._uguiReady && _driver._dfCanvas != null)
                {
                    _driver._dfCanvas.ShowToast(message, toastType);
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[HmrUiActionsAdapter] ShowToast failed: {ex.Message}");
            }
        }

        /// <inheritdoc/>
        public void ShowConfirmDialog(string message, Action onConfirm, Action onCancel)
        {
            try
            {
                UI.ModMenuPanel? panel = _driver._dfCanvas?.ModMenuPanel;
                if (panel != null)
                {
                    panel.ShowConfirmDialog(message, onConfirm, onCancel);
                }
                else
                {
                    // No panel available — auto-cancel so we never silently block.
                    DebugLog.Write("Plugin", "[HmrUiActionsAdapter] ShowConfirmDialog: ModMenuPanel unavailable, auto-cancelling.");
                    onCancel?.Invoke();
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[HmrUiActionsAdapter] ShowConfirmDialog failed: {ex.Message}");
                onCancel?.Invoke();
            }
        }
    }
}
