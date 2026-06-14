#nullable enable
using System;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Threading;
using HarmonyLib;
using Unity.Entities;
using UnityEngine;
using UnityEngine.LowLevel;
using DINOForge.Runtime;
using DINOForge.Runtime.Diagnostics;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// ECS system that handles F9/F10 key input and owns the IMGUI overlay.
    /// ECS systems survive DINO's scene transitions (unlike MonoBehaviours).
    ///
    /// Placed in SimulationSystemGroup with [AlwaysUpdateSystem] so it ticks
    /// even at the main menu before game entities load. Without SimulationSystemGroup,
    /// InitializationSystemGroup may not be created/ticked by DINO's ECS setup.
    ///
    /// IMGUI strategy: attach DebugOverlayBehaviour to DINO's own main camera
    /// (which DINO keeps alive across transitions). We piggyback on their camera
    /// rather than creating our own GameObject that DINO will destroy.
    /// </summary>
    [AlwaysUpdateSystem]
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class KeyInputSystem : SystemBase
    {
        // Win32 P/Invoke for key detection
        // Input.GetKey() relies on MonoBehaviour.Update() polling, which NEVER fires in DINO.
        // We must use GetAsyncKeyState() directly to work with DINO's custom PlayerLoop.
        [DllImport("user32.dll")]
        private static extern ushort GetAsyncKeyState(int vKey);

        private const int VK_F9 = 0x78;
        private const int VK_F10 = 0x79;
        private const ushort KEY_PRESSED = 0x8000;
        /// <summary>
        /// Caches the world that KeyInputSystem lives in. Updated on every OnCreate.
        /// Used by GameBridgeServer to always query the correct world after scene transitions.
        /// </summary>
        private static World? _cachedWorld;

        /// <summary>Cached world name updated by OnUpdate. Thread-safe string reads.</summary>
        private static string _lastCachedWorldName = "";

        /// <summary>
        /// Cached entity count updated by OnUpdate.
        /// Thread-safe read (int reads are atomic in .NET).
        /// Read by GameBridgeServer.HandleStatus to avoid main-thread dispatch from background thread.
        /// </summary>
        private static int _lastCachedEntityCount;

        /// <summary>Returns cached entity count from OnUpdate. Returns -2 if never updated.</summary>
        public static int LastEntityCount => _lastCachedEntityCount;

        /// <summary>Returns cached world name from OnUpdate.</summary>
        public static string CachedWorldName => _lastCachedWorldName;

        /// <summary>
        /// Returns the ECS world that the active KeyInputSystem instance lives in.
        /// Falls back to World.DefaultGameObjectInjectionWorld if no instance exists.
        /// </summary>
        public static World? GetActiveWorld()
        {
            World? result = _cachedWorld ?? World.DefaultGameObjectInjectionWorld;
            // If cached/default world is null or disposed, scan all worlds for a valid one.
            if (result == null || !result.IsCreated)
            {
                DebugLog.Write("KeyInput", "[KeyInputSystem.GetActiveWorld] cached/default world invalid — scanning all worlds...");
                foreach (World w in World.All)
                {
                    if (w.IsCreated)
                    {
                        result = w;
                        DebugLog.Write("KeyInput", $"[KeyInputSystem.GetActiveWorld] Found valid world: '{w.Name}'.");
                        break;
                    }
                }
            }
            return result;
        }

        /// <summary>Called when F9 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF9Pressed;

        /// <summary>Called when F10 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF10Pressed;

        /// <summary>Called when pack reload is requested (set by RuntimeDriver if alive). Can be invoked from background thread.</summary>
        public static System.Action? OnPackReloadRequested;

        // ── P0 fix: Win32 background polling thread ─────────────────────────────────
        // CLAUDE.md claims F9/F10 work via a Win32 GetAsyncKeyState background thread.
        // Reality (pre-fix): polling lived in OnUpdate, which only ticks inside
        // SimulationSystemGroup — DORMANT at the main menu. Result: F9/F10 don't fire
        // until the user has actually started a game, making the debug overlay
        // unreachable from the menu.
        //
        // This thread polls GetAsyncKeyState every 50ms regardless of ECS group state,
        // so F9/F10 fire from the moment Plugin.Awake completes. Per CLAUDE.md, invoking
        // SetActive / OnF9Pressed from a background thread is safe in Mono 2021.3 for
        // DontDestroyOnLoad objects. OnUpdate polling is kept as a redundant backup.
        private static Thread? _keyPollThread;
        private static volatile bool _keyPollRunning;
        private static bool _bgF9PreviousState;
        private static bool _bgF10PreviousState;

        /// <summary>
        /// Starts the background Win32 key-polling thread. Idempotent — safe to call
        /// multiple times. Must be called from Plugin.Awake so F9/F10 work at the main
        /// menu (before SimulationSystemGroup begins ticking).
        /// </summary>
        public static void StartKeyPollThread()
        {
            if (_keyPollRunning) return;
            _keyPollRunning = true;
            _keyPollThread = new Thread(KeyPollLoop)
            {
                IsBackground = true,
                Name = "DINOForge-F9F10-Hook",
            };
            _keyPollThread.Start();
            DebugLog.Write("KeyInput", "[KeyInputSystem.StartKeyPollThread] Background F9/F10 poll thread started.");
        }

        /// <summary>
        /// Stops the background polling thread cleanly. Called from Plugin.OnDestroy.
        /// </summary>
        public static void StopKeyPollThread()
        {
            _keyPollRunning = false;
            _keyPollThread = null;
            DebugLog.Write("KeyInput", "[KeyInputSystem.StopKeyPollThread] Background F9/F10 poll thread stopped.");
        }

        private static void KeyPollLoop()
        {
            while (_keyPollRunning)
            {
                try
                {
                    bool f9Now = (GetAsyncKeyState(VK_F9) & KEY_PRESSED) != 0;
                    bool f10Now = (GetAsyncKeyState(VK_F10) & KEY_PRESSED) != 0;
                    if (f9Now && !_bgF9PreviousState)
                    {
                        DebugLog.Write("KeyInput", "[KeyPollLoop] F9 edge detected (background thread).");
                        try { OnF9Pressed?.Invoke(); } catch (Exception ex) { DebugLog.Write("KeyInput", $"[KeyPollLoop] OnF9Pressed threw: {ex.Message}"); }
                    }
                    _bgF9PreviousState = f9Now;
                    if (f10Now && !_bgF10PreviousState)
                    {
                        DebugLog.Write("KeyInput", "[KeyPollLoop] F10 edge detected (background thread).");
                        try { OnF10Pressed?.Invoke(); } catch (Exception ex) { DebugLog.Write("KeyInput", $"[KeyPollLoop] OnF10Pressed threw: {ex.Message}"); }
                    }
                    _bgF10PreviousState = f10Now;
                }
                catch (Exception ex)
                {
                    /* safe-swallow: best-effort background polling must continue even if one iteration fails; logged below. */
                    try { DebugLog.Write("KeyInput", $"[KeyPollLoop] iter exception: {ex.Message}"); }
                    catch (Exception logEx) { DebugLog.Write("KeyInput", $"[KeyPollLoop] logging failed: {logEx.Message}"); }
                }
                try { Thread.Sleep(50); }
                catch (ThreadInterruptedException ex)
                {
                    DebugLog.Write("KeyInput", $"[KeyPollLoop] sleep interrupted: {ex.Message}; exiting poll loop.");
                    break;
                }
            }
        }

        /// <summary>
        /// Cached entity count updated by OnUpdate every tick.
        /// Thread-safe read (int reads are atomic in .NET).
        private bool _overlayEnsured;
        private int _updateFrame;
        private bool _f9PreviousState;
        private bool _f10PreviousState;
        // Tracks the last DefaultGameObjectInjectionWorld seen by OnUpdate.
        // When it changes (scene transition), we re-check if KeyInputSystem is in the right world.
        private World? _lastDefaultWorld;

        private static bool _keyLoopHarmonyPatched;

        /// <summary>
        /// Returns the ECS world that this KeyInputSystem instance lives in.
        /// Used by GameBridgeServer to query the correct world (which may differ from
        /// World.DefaultGameObjectInjectionWorld after scene transitions).
        /// </summary>
        public World OwningWorld => World;

#if NET8_0
        public override void OnCreate()
#else
        protected override void OnCreate()
#endif
        {
            DebugLog.Write("KeyInput", $"KeyInputSystem.OnCreate: World='{World?.Name ?? "null"}' IsCreated={World?.IsCreated ?? false}");
            Enabled = true;
            // Attempt resurrection in OnCreate — this fires when a new ECS world starts,
            // which happens after DINO tears down the previous world (and our RuntimeDriver).
            //
            // IMPORTANT: We defer TryResurrect to the background polling thread instead of calling
            // it directly. This is because when a new RuntimeDriver is created during scene
            // transition, its Plugin.Awake() hasn't completed yet. Calling TryResurrect directly
            // would fail because _resurrectionLog/_resurrectionConfig are null.
            if (Plugin.NeedsResurrection || ReferenceEquals(Plugin.PersistentRoot, null))
            {
                DebugLog.Write("KeyInput", $"[KeyInputSystem.OnCreate] Resurrection needed: NeedsRes={Plugin.NeedsResurrection} rootRef={(!ReferenceEquals(Plugin.PersistentRoot, null))} paramsReady={Plugin.ResurrectionParamsReady}");
                // Keep the need MARKED (do NOT clear NeedsResurrection prematurely — clearing it here
                // was a stale-event clear of the flag that the WinDbg-diagnosed dormant-plugin bug
                // relied on the fallback to re-observe).
                Plugin.NeedsDeferredResurrection = true;

                // iter-149e DECISIVE fix: OnCreate fires on the Unity MAIN THREAD when DINO creates a
                // new ECS world — and DINO creates one while bringing up MainMenu (the InitialGameLoader
                // 'Default World' OnCreate proves this callback fires post-teardown). After the
                // InitialGameLoader->MainMenu teardown, our bg fallback thread goes silent and no
                // MainMenu sceneLoaded/activeSceneChanged reaches our static handlers, so this
                // DINO-driven main-thread callback is the reliable revive hook. Revive DIRECTLY here
                // (main-thread ECalls are safe) once Awake captured the resurrection params. The
                // original "defer only" rationale (Awake not yet complete) only holds for the very
                // first world during startup; ResurrectionParamsReady gates that.
                if (Plugin.ResurrectionParamsReady)
                {
                    Plugin.ReviveFromMainThreadCallback("KeyInputSystem.OnCreate(main-thread)");
                }
            }

            // Key insight: OnCreate fires BEFORE World.DefaultGameObjectInjectionWorld is set,
            // so this system ends up in whatever world DINO created, NOT the default world.
            // We cache our world here so GameBridgeServer can always query the correct world.
            _cachedWorld = World;
            _lastDefaultWorld = World.DefaultGameObjectInjectionWorld;

            // SPEC-004 Path 3: independent PlayerLoop marker + SetPlayerLoop re-injection.
            try
            {
                InjectIntoPlayerLoop();
                PatchPlayerLoopSetPlayerLoop();
            }
            catch (Exception ex)
            {
                DebugLog.Write("KeyInput", $"[KeyInputSystem.OnCreate] PlayerLoop inject failed: {ex.Message}");
            }

            DebugLog.Write("KeyInput", $"KeyInputSystem.OnCreate complete, Enabled={Enabled}");
        }

        /// <summary>SPEC-004 Path 3: append <see cref="PlayerLoopKeyInputInjection.DINOForgeKeyLoopMarker"/> to PlayerLoop.Update.</summary>
        private static void InjectIntoPlayerLoop()
        {
            PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                typeof(PlayerLoopKeyInputInjection.DINOForgeKeyLoopMarker),
                DINOForgeKeyLoopUpdate);
        }

        private static void DINOForgeKeyLoopUpdate()
        {
            if (Input.GetKeyDown(KeyCode.F9))
            {
                OnF9Pressed?.Invoke();
            }

            if (Input.GetKeyDown(KeyCode.F10))
            {
                OnF10Pressed?.Invoke();
            }
        }

        private static void PatchPlayerLoopSetPlayerLoop()
        {
            if (_keyLoopHarmonyPatched)
            {
                return;
            }

            try
            {
                var harmony = new Harmony("dinoforge.keyinput.playerloop");
                MethodInfo? original = typeof(PlayerLoop).GetMethod(
                    nameof(PlayerLoop.SetPlayerLoop),
                    BindingFlags.Public | BindingFlags.Static);
                if (original == null)
                {
                    DebugLog.Write("KeyInput", "[KeyInputSystem] PatchPlayerLoopSetPlayerLoop: SetPlayerLoop not found.");
                    return;
                }

                MethodInfo? postfix = typeof(KeyInputSystem).GetMethod(
                    nameof(OnKeyInputPlayerLoopSet),
                    BindingFlags.NonPublic | BindingFlags.Static);
                harmony.Patch(original, postfix: new HarmonyMethod(postfix));
                _keyLoopHarmonyPatched = true;
                DebugLog.Write("KeyInput", "[KeyInputSystem] Harmony postfix on PlayerLoop.SetPlayerLoop applied.");
            }
            catch (Exception ex)
            {
                DebugLog.Write("KeyInput", $"[KeyInputSystem] PatchPlayerLoopSetPlayerLoop failed: {ex.Message}");
            }
        }

        private static void OnKeyInputPlayerLoopSet()
        {
            PlayerLoopKeyInputInjection.OnAfterSetPlayerLoop(() =>
                PlayerLoopKeyInputInjection.InjectIntoCurrentPlayerLoop(
                    typeof(PlayerLoopKeyInputInjection.DINOForgeKeyLoopMarker),
                    DINOForgeKeyLoopUpdate));
        }

#if NET8_0
        public override void OnDestroy()
#else
        protected override void OnDestroy()
#endif
        {
            DebugLog.Write("KeyInput", $"KeyInputSystem.OnDestroy: World='{World?.Name ?? "null"}' IsCreated={World?.IsCreated ?? false}");
            // Clear the cached world reference so GetActiveWorld() falls back to scanning
            // all worlds until a new KeyInputSystem is registered in the new world.
            _cachedWorld = null;

            // CRITICAL (task #535): mark the dispatcher pump dead so background-thread
            // bridge handlers fail fast instead of queueing work that can never be
            // drained. Without this, every .Result / .Wait(...) in GameBridgeServer
            // burns its full timeout once KeyInputSystem is gone, accumulating
            // wedged operations until the bridge thread parks indefinitely
            // (IsHungAppWindow=True). The flag is re-armed automatically on the
            // next DrainQueue tick once a fresh KeyInputSystem is registered.
            MainThreadDispatcher.MarkPumpDead();

            base.OnDestroy();
        }

#if NET8_0
        public override void OnUpdate()
#else
        protected override void OnUpdate()
#endif
        {
            try
            {
                _updateFrame++;
                // Log every 600 frames (once per ~10 seconds at 60 FPS)
                if (_updateFrame % 600 == 0)
                    DebugLog.Write("KeyInput", $"[KeyInputSystem.OnUpdate] frame={_updateFrame} enabled={Enabled} overlayEnsured={_overlayEnsured} PersistentRoot={(Plugin.PersistentRoot != null ? "alive" : "null")}");

                // Drain the MainThreadDispatcher queue from ECS OnUpdate.
                // MonoBehaviour.Update() never fires in DINO (custom PlayerLoop),
                // so this is the only reliable pump for main-thread work.
                MainThreadDispatcher.DrainQueue();

                // Ensure bridge server thread is alive — may have been aborted during
                // scene transitions. Restart it if dead so CLI/MCP tools recover.
                Plugin.SharedBridgeServer?.EnsureServerAlive();

                // If PersistentRoot was destroyed by DINO, resurrect it via ECS.
                // Only call TryResurrect when resurrection is actually needed to avoid
                // spamming RecreateInCurrentWorld on every ECS tick.
                if (Plugin.NeedsResurrection || Plugin.NeedsDeferredResurrection)
                {
                    bool wasDeferred = Plugin.NeedsDeferredResurrection;
                    Plugin.NeedsResurrection = false;
                    Plugin.NeedsDeferredResurrection = false;
                    Plugin.TryResurrect(wasDeferred ? "(deferred)" : "(ECS tick)", "KeyInputSystem");
                }

                // Detect world changes (scene transitions) and re-register KeyInputSystem
                // in DefaultGameObjectInjectionWorld if it changed. This fixes the bug where
                // OnCreate fires before DefaultGameObjectInjectionWorld is set, causing the
                // system to be registered in the wrong world and DrainQueue to never run.
                World? currentDefault = World.DefaultGameObjectInjectionWorld;
                if (currentDefault != null && !ReferenceEquals(currentDefault, _lastDefaultWorld))
                {
                    DebugLog.Write("KeyInput", $"[KeyInputSystem.OnUpdate] DefaultGameObjectInjectionWorld changed: " +
                        $"'{_lastDefaultWorld?.Name ?? "null"}' → '{currentDefault.Name}'. " +
                        $"Re-registering in new world.");
                    try
                    {
                        currentDefault.GetOrCreateSystem<KeyInputSystem>();
                        DebugLog.Write("KeyInput", "[KeyInputSystem.OnUpdate] KeyInputSystem registered in new default world.");
                    }
                    catch (Exception ex)
                    {
                        DebugLog.Write("KeyInput", $"[KeyInputSystem.OnUpdate] Re-registration failed: {ex.Message}");
                    }
                    _lastDefaultWorld = currentDefault;
                }

                // Consume any pending F9/F10 toggles (for future compatibility)
                if (Plugin.PendingF9Toggle)
                {
                    Plugin.PendingF9Toggle = false;
                    DebugLog.Write("KeyInput", "Consumed PendingF9Toggle");
                }
                if (Plugin.PendingF10Toggle)
                {
                    Plugin.PendingF10Toggle = false;
                    DebugLog.Write("KeyInput", "Consumed PendingF10Toggle");
                }

                // Ensure overlay component is attached to a surviving GameObject
                if (!_overlayEnsured)
                    EnsureOverlay();

                // Poll Win32 key state for F9/F10 — detect PRESS (key goes from up to down), not hold
                // NOTE: Input.GetKey() uses MonoBehaviour.Update() which NEVER fires in DINO.
                // GetAsyncKeyState() works with DINO's custom PlayerLoop.
                bool f9Current = (GetAsyncKeyState(VK_F9) & KEY_PRESSED) != 0;
                bool f10Current = (GetAsyncKeyState(VK_F10) & KEY_PRESSED) != 0;

                // F9: trigger on transition from not-pressed to pressed
                if (f9Current && !_f9PreviousState)
                {
                    DebugLog.Write("KeyInput", "F9 pressed (transition detected)");
                    if (OnF9Pressed != null)
                        OnF9Pressed.Invoke();
                    else
                        DebugOverlayBehaviour.Instance?.Toggle();
                }
                _f9PreviousState = f9Current;

                // F10: trigger on transition from not-pressed to pressed
                if (f10Current && !_f10PreviousState)
                {
                    DebugLog.Write("KeyInput", "F10 pressed (transition detected)");
                    OnF10Pressed?.Invoke();
                }
                _f10PreviousState = f10Current;

                // Cache entity count for background-thread readers (GameBridgeServer).
                // Cache world name and entity count for background-thread readers (GameBridgeServer).
                // World name is a string (thread-safe), entity count is int (atomic read).
                try
                {
                    World? w = _cachedWorld ?? World;
                    // If our cached world is invalid, find any valid world.
                    if (w == null || !w.IsCreated)
                    {
                        foreach (World candidate in World.All)
                        {
                            if (candidate.IsCreated) { w = candidate; break; }
                        }
                    }
                    _lastCachedWorldName = (w != null && w.IsCreated) ? (w.Name ?? "") : "";
                    if (w != null && w.IsCreated)
                    {
                        EntityQuery all = w.EntityManager.CreateEntityQuery(new EntityQueryDesc
                        {
                            Options = EntityQueryOptions.IncludePrefab | EntityQueryOptions.IncludeDisabled
                        });
                        _lastCachedEntityCount = all.CalculateEntityCount();
                        all.Dispose();
                    }
                    else
                    {
                        _lastCachedEntityCount = -1;
                    }
                }
                catch
                {
                    _lastCachedWorldName = "";
                    _lastCachedEntityCount = -1;
                }
            }
            catch (System.Exception ex)
            {
                DebugLog.Write("KeyInput", $"KeyInputSystem.OnUpdate EXCEPTION: {ex.GetType().Name}: {ex.Message}\n{ex.StackTrace}");
            }
        }

        /// <summary>
        /// Called by the bridge supervisor on a background thread when it detects that
        /// our owning world was destroyed (RuntimeDriver.OnDestroy). Recreates KeyInputSystem
        /// in the current DefaultGameObjectInjectionWorld so the pump survives scene transitions.
        /// This is the only reliable way to handle world changes — OnUpdate only runs while
        /// the system is alive, so it can't self-recover after destruction.
        /// </summary>
        public static void RecreateInCurrentWorld()
        {
            try
            {
                // Find the current ECS world: prefer DefaultGameObjectInjectionWorld if valid,
                // otherwise scan all worlds and pick the first one with entities (handles scene
                // transitions where DefaultGameObjectInjectionWorld may lag behind).
                World? current = World.DefaultGameObjectInjectionWorld;
                if (current == null || !current.IsCreated)
                {
                    DebugLog.Write("KeyInput", "[KeyInputSystem.RecreateInCurrentWorld] DefaultWorld null/disposed — scanning all worlds...");
                    foreach (World w in World.All)
                    {
                        if (w.IsCreated)
                        {
                            current = w;
                            DebugLog.Write("KeyInput", $"[KeyInputSystem.RecreateInCurrentWorld] Found valid world: '{w.Name}'.");
                            break;
                        }
                    }
                }
                if (current == null || !current.IsCreated)
                {
                    DebugLog.Write("KeyInput", "[KeyInputSystem.RecreateInCurrentWorld] No valid world found.");
                    return;
                }
                DebugLog.Write("KeyInput", $"[KeyInputSystem.RecreateInCurrentWorld] Calling GetOrCreateSystem in '{current.Name}' (IsCreated={current.IsCreated}).");
                KeyInputSystem sys = current.GetOrCreateSystem<KeyInputSystem>();
                DebugLog.Write("KeyInput", $"[KeyInputSystem.RecreateInCurrentWorld] Got system: World={sys.World?.Name ?? "null"} IsCreated={sys.World?.IsCreated ?? false}");
                // Update the cached world so GetActiveWorld() returns the current world.
                _cachedWorld = current;
                DebugLog.Write("KeyInput", $"[KeyInputSystem.RecreateInCurrentWorld] Registered in '{current.Name}'.");
            }
            catch (Exception ex)
            {
                DebugLog.Write("KeyInput", $"[KeyInputSystem.RecreateInCurrentWorld] Failed: {ex.Message}\n{ex.StackTrace}");
            }
        }

        private void EnsureOverlay()
        {
            if (DebugOverlayBehaviour.Instance != null)
            {
                _overlayEnsured = true;
                return;
            }

            // Try to piggyback on DINO's main camera — DINO keeps it alive
            Camera? cam = Camera.main;
            if (cam == null)
            {
                // Camera not ready yet — try all cameras
                Camera[] cams = Camera.allCameras;
                if (cams.Length > 0) cam = cams[0];
            }

            if (cam != null)
            {
                cam.gameObject.AddComponent<DebugOverlayBehaviour>();
                _overlayEnsured = true;
                DebugLog.Write("KeyInput", $"EnsureOverlay: attached DebugOverlayBehaviour to camera '{cam.name}'");
            }
        }

    }
}
