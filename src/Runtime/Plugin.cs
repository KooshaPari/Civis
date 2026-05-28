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

        /// <summary>Flag set by KeyInputSystem when F9 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF9Toggle;

        /// <summary>Flag set by KeyInputSystem when F10 is pressed during ECS tick.</summary>
        internal static volatile bool PendingF10Toggle;

        /// <summary>Flag indicating PersistentRoot needs resurrection.</summary>
        internal static volatile bool NeedsResurrection;

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

            _showOverlayOnStart = showOverlayOnStart;
            _enableHotReload = enableHotReload;
            _hmrDebounceMs = hmrDebounceMs;

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
                Bridge.ResourcesUnloadGuardPatch.Apply(_harmony);
                Bridge.AssetBundleUnloadGuardPatch.Apply(_harmony);
                Bridge.AssetBundleLoadGuardPatch.Apply(_harmony);
                Bridge.SceneUnloadGuardPatch.Apply(_harmony);
                Bridge.WorldDisposeGuardPatch.Apply(_harmony);
                UI.ModsButtonTextPatch.Apply(_harmony);
                Log.LogInfo("Harmony initialized and patches applied.");
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
            DebugLog.Write("Plugin", "[Plugin] activeSceneChanged watcher registered (iter-144 #546 fix).");
            StartResurrectionFallbackThread();
        }

        private static void OnActiveSceneChanged(Scene oldScene, Scene newScene)
        {
            DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged: old='{oldScene.name}' new='{newScene.name}'");
            // Iter-144 menu-unclickable fix: DINO's MainMenu scene EventSystem is destroyed on
            // scene transitions, leaving EventSystem.current = null even though our
            // DontDestroyOnLoad'd EventSystem (DFCanvas) still exists. Re-promote (or recreate)
            // on every scene change so NativeMenuInjector clicks route correctly.
            EnsureEventSystemAlive();
            try
            {
                Bridge.KeyInputSystem.RecreateInCurrentWorld();
            }
            catch (Exception ex)
            {
                DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged RecreateInCurrentWorld failed: {ex.Message}");
            }
            // RuntimeDriver may have been destroyed when DINO destroyed our root.
            // Trigger resurrection here. IMPORTANT: we defer TryResurrect to the resurrection thread
            // (or the new RuntimeDriver's BG poll thread) instead of calling directly, since a brand
            // new RuntimeDriver may not have completed Initialize() yet at this exact tick.
            // Iter-144 #543 fix: ReferenceEquals check + s_rootJustDestroyed avoids Unity fake-null
            // trap (== null on a destroyed-but-not-nulled MonoBehaviour reports true via operator
            // overload but the actual managed reference is still non-null).
            bool rootIsRefNull = ReferenceEquals(PersistentRoot, null);
            if (NeedsResurrection || s_rootJustDestroyed || rootIsRefNull || PersistentRoot == null)
            {
                DebugLog.Write("Plugin", $"[Plugin] OnActiveSceneChanged: resurrection needed - NeedsRes={NeedsResurrection} rootJustDestroyed={s_rootJustDestroyed} refNull={rootIsRefNull} unityNull={PersistentRoot == null}");
                LastSceneNameForResurrection = newScene.name;
                NeedsDeferredResurrection = true;
                DebugLog.Write("Plugin", "[Plugin] Resurrection complete via activeSceneChanged (flagged for deferred TryResurrect)");
            }
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
            DateTime lastNeedsObservedUtc = DateTime.MinValue;
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

                    // GameLaunch attach-mode: KeyInputSystem may be absent from the ECS world while the
                    // plugin and PersistentRoot survive scene transitions. Restart the bridge pipe when
                    // its server thread died (BridgeServerThreadAlive=False after OnDestroy).
                    try
                    {
                        SharedBridgeServer?.EnsureServerAlive();
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback EnsureServerAlive: {ex.Message}");
                    }

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
                        lastNeedsObservedUtc = DateTime.MinValue;
                        continue;
                    }
                    if (lastNeedsObservedUtc == DateTime.MinValue)
                    {
                        lastNeedsObservedUtc = DateTime.UtcNow;
                        DebugLog.Write("Plugin", "[Plugin] ResurrectionFallback: NeedsResurrection observed, starting grace timer.");
                        continue;
                    }
                    TimeSpan since = DateTime.UtcNow - lastNeedsObservedUtc;
                    if (since.TotalMilliseconds < GraceWindowMs) continue;
                    // Grace window exceeded with no scene-event resolution: attempt direct resurrect.
                    if (_resurrectionLog == null || _resurrectionConfig == null)
                    {
                        // Plugin.Awake never completed; can't resurrect. Reset timer to retry later.
                        DebugLog.Write("Plugin", "[Plugin] ResurrectionFallback: cannot revive (Plugin.Awake state not captured). Will retry.");
                        lastNeedsObservedUtc = DateTime.UtcNow;
                        continue;
                    }
                    string sceneName = LastSceneNameForResurrection ?? "fallback-unknown";
                    DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback: grace window {GraceWindowMs}ms exceeded — invoking TryResurrect (scene='{sceneName}').");
                    try
                    {
                        TryResurrect(sceneName, "ResurrectionFallbackThread");
                        // After attempt, clear flags so we don't spin; if revive failed, scene event/poller will re-set them.
                        NeedsResurrection = false;
                        NeedsDeferredResurrection = false;
                        s_rootJustDestroyed = false;
                        s_skipBundleUnload = false;
                        DebugLog.Write("Plugin", "[Plugin] Resurrection complete via ResurrectionFallbackThread (flags cleared).");
                        lastNeedsObservedUtc = DateTime.MinValue;
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("Plugin", $"[Plugin] ResurrectionFallback TryResurrect threw: {ex.Message}");
                        lastNeedsObservedUtc = DateTime.UtcNow; // back off, retry next grace window
                    }
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
                EnsureEventSystemAlive();
                try { SharedBridgeServer?.EnsureServerAlive(); }
                catch (Exception ex) { DebugLog.Write("Plugin", $"[PlayerLoop] EnsureServerAlive: {ex.Message}"); }
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
            Bridge.PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(() =>
                Bridge.PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                    typeof(Bridge.PlayerLoopKeyInputInjection.DINOForgeUpdateMarker),
                    DINOForgePlayerLoopUpdate));
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

        // Loading overlay (shown during mod init, hidden when scene loads)
        private ModLoadingOverlay? _loadingOverlay;

        // Active UI hosts.
        // _modMenuHost is always set to the active menu (UGUI when healthy, IMGUI fallback otherwise).
        // _debugOverlay is ALWAYS added (it owns the IMGUI F9 debug panel).
        private IModMenuHost? _modMenuHost;
        private IModSettingsHost? _modSettingsHost;
        private DebugOverlayBehaviour? _debugOverlay;
        private HudIndicator? _hudIndicator;
        private NativeMenuInjector? _nativeMenuInjector;
        private MainMenuThemer? _mainMenuThemer;

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
                Bridge.KeyInputSystem.OnF9Pressed = () =>
                {
                    try
                    {
                        DebugLog.Write("Plugin", "[RuntimeDriver] F9 pressed (via KeyInputSystem)");
                        if (_uguiReady && _dfCanvas != null) _dfCanvas.ToggleModMenu();
                        else _modMenuHost?.Toggle();
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
                        DebugLog.Write("Plugin", "[RuntimeDriver] F10 pressed (via KeyInputSystem)");
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

            // ── Step 6.5: Create loading overlay ────────────────────────────────────
            // Show a skeleton UI during the ~30-45s mod initialization phase.
            // This overlay is hidden when the MainMenu scene fully loads.
            RunPhaseWithAbortGuard("ModLoadingOverlay.Create", () =>
            {
                try
                {
                    _loadingOverlay = ModLoadingOverlay.Create(gameObject);
                    if (_loadingOverlay != null)
                    {
                        _log.LogInfo("[RuntimeDriver] ModLoadingOverlay created.");
                    }
                }
                catch (Exception ex)
                {
                    _log.LogWarning($"[RuntimeDriver] ModLoadingOverlay creation failed: {ex}");
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
                if (_modPlatform != null)
                {
                    _log.LogInfo("[RuntimeDriver] MainMenu-mode pack-load: calling LoadPacks() (no ECS world required).");
                    try
                    {
                        var result = _modPlatform.LoadPacks();
                        _log.LogInfo($"[RuntimeDriver] MainMenu-mode pack-load complete: success={result.IsSuccess}, loaded={result.LoadedPacks.Count}, errors={result.Errors.Count}");
                        WireUguiToModPlatform();
                        PushLoadedPacksToUgui("main-menu init");

                        // Hide loading overlay now that packs are loaded
                        if (_loadingOverlay != null)
                        {
                            _loadingOverlay.Hide();
                            _log.LogInfo("[RuntimeDriver] ModLoadingOverlay hidden (MainMenu-mode pack-load complete).");
                        }

                        // Apply total_conversion theme to main menu
                        try
                        {
                            _mainMenuThemer = new MainMenuThemer(_log, _modPlatform.PacksDirectory);
                            var packInfos = _modPlatform.GetLoadedPackDisplayInfos();
                            _mainMenuThemer.TryApplyTheme(packInfos);
                        }
                        catch (Exception themeEx)
                        {
                            _log.LogWarning($"[RuntimeDriver] MainMenuThemer failed: {themeEx.Message}");
                        }
                    }
                    catch (Exception ex)
                    {
                        _log.LogError($"[RuntimeDriver] MainMenu-mode pack-load FAILED: {ex}");
                    }
                }
                else
                {
                    _log.LogWarning("[RuntimeDriver] MainMenu-mode pack-load skipped — _modPlatform is null.");
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

                    // Hide the loading overlay now that world is ready and packs are loaded
                    if (_loadingOverlay != null)
                    {
                        _loadingOverlay.Hide();
                        _log?.LogInfo("[RuntimeDriver] ModLoadingOverlay hidden (world ready).");
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
            ContextualModMenuHost contextualHost = new ContextualModMenuHost(
                _dfCanvas.ModMenuPanel, nativeHost);
            _nativeMenuInjector.SetModMenuHost(contextualHost);
            _log?.LogInfo("[RuntimeDriver] NativeMenuInjector wired via ContextualModMenuHost (native stub active, overlay fallback).");
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
