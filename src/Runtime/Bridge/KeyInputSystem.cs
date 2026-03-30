#nullable enable
using System;
using System.Reflection;
using Unity.Entities;
using UnityEngine;
using DINOForge.Runtime;

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
                WriteDebug("[KeyInputSystem.GetActiveWorld] cached/default world invalid — scanning all worlds...");
                foreach (World w in World.All)
                {
                    if (w.IsCreated)
                    {
                        result = w;
                        WriteDebug($"[KeyInputSystem.GetActiveWorld] Found valid world: '{w.Name}'.");
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

        /// <summary>
        /// Returns the ECS world that this KeyInputSystem instance lives in.
        /// Used by GameBridgeServer to query the correct world (which may differ from
        /// World.DefaultGameObjectInjectionWorld after scene transitions).
        /// </summary>
        public World OwningWorld => World;

        protected override void OnCreate()
        {
            WriteDebug($"KeyInputSystem.OnCreate: World='{World?.Name ?? "null"}' IsCreated={World?.IsCreated ?? false}");
            Enabled = true;
            // Attempt resurrection in OnCreate — this fires when a new ECS world starts,
            // which happens after DINO tears down the previous world (and our RuntimeDriver).
            if (Plugin.NeedsResurrection || ReferenceEquals(Plugin.PersistentRoot, null))
            {
                WriteDebug($"[KeyInputSystem.OnCreate] Resurrection needed: NeedsRes={Plugin.NeedsResurrection} rootRef={(!ReferenceEquals(Plugin.PersistentRoot, null))}");
                Plugin.NeedsResurrection = false;
                Plugin.TryResurrect("(ECS OnCreate)", "KeyInputSystem.OnCreate");
            }

            // Key insight: OnCreate fires BEFORE World.DefaultGameObjectInjectionWorld is set,
            // so this system ends up in whatever world DINO created, NOT the default world.
            // We cache our world here so GameBridgeServer can always query the correct world.
            _cachedWorld = World;
            _lastDefaultWorld = World.DefaultGameObjectInjectionWorld;

            WriteDebug($"KeyInputSystem.OnCreate complete, Enabled={Enabled}");
        }

        protected override void OnDestroy()
        {
            WriteDebug($"KeyInputSystem.OnDestroy: World='{World?.Name ?? "null"}' IsCreated={World?.IsCreated ?? false}");
            // Clear the cached world reference so GetActiveWorld() falls back to scanning
            // all worlds until a new KeyInputSystem is registered in the new world.
            _cachedWorld = null;
            base.OnDestroy();
        }

        protected override void OnUpdate()
        {
            try
            {
                _updateFrame++;
                // Log every frame for first 5 frames, then every 600
                if (_updateFrame <= 5 || _updateFrame % 600 == 0)
                    WriteDebug($"[KeyInputSystem.OnUpdate] frame={_updateFrame} enabled={Enabled} overlayEnsured={_overlayEnsured} PersistentRoot={(Plugin.PersistentRoot != null ? "alive" : "null")}");

                // Drain the MainThreadDispatcher queue from ECS OnUpdate.
                // MonoBehaviour.Update() never fires in DINO (custom PlayerLoop),
                // so this is the only reliable pump for main-thread work.
                MainThreadDispatcher.DrainQueue();

                // Ensure bridge server thread is alive — may have been aborted during
                // scene transitions. Restart it if dead so CLI/MCP tools recover.
                Plugin.SharedBridgeServer?.EnsureServerAlive();

                // If PersistentRoot was destroyed by DINO, resurrect it via ECS
                Plugin.TryResurrect("(ECS tick)", "KeyInputSystem");

                // Detect world changes (scene transitions) and re-register KeyInputSystem
                // in DefaultGameObjectInjectionWorld if it changed. This fixes the bug where
                // OnCreate fires before DefaultGameObjectInjectionWorld is set, causing the
                // system to be registered in the wrong world and DrainQueue to never run.
                World? currentDefault = World.DefaultGameObjectInjectionWorld;
                if (currentDefault != null && !ReferenceEquals(currentDefault, _lastDefaultWorld))
                {
                    WriteDebug($"[KeyInputSystem.OnUpdate] DefaultGameObjectInjectionWorld changed: " +
                        $"'{_lastDefaultWorld?.Name ?? "null"}' → '{currentDefault.Name}'. " +
                        $"Re-registering in new world.");
                    try
                    {
                        currentDefault.GetOrCreateSystem<KeyInputSystem>();
                        WriteDebug("[KeyInputSystem.OnUpdate] KeyInputSystem registered in new default world.");
                    }
                    catch (Exception ex)
                    {
                        WriteDebug($"[KeyInputSystem.OnUpdate] Re-registration failed: {ex.Message}");
                    }
                    _lastDefaultWorld = currentDefault;
                }

                // Consume any pending F9/F10 toggles (for future compatibility)
                if (Plugin.PendingF9Toggle)
                {
                    Plugin.PendingF9Toggle = false;
                    WriteDebug("Consumed PendingF9Toggle");
                }
                if (Plugin.PendingF10Toggle)
                {
                    Plugin.PendingF10Toggle = false;
                    WriteDebug("Consumed PendingF10Toggle");
                }

                // Ensure overlay component is attached to a surviving GameObject
                if (!_overlayEnsured)
                    EnsureOverlay();

                // Poll Unity Input for F9/F10 — detect PRESS (key goes from up to down), not hold
                bool f9Current = Input.GetKey(KeyCode.F9);
                bool f10Current = Input.GetKey(KeyCode.F10);

                // F9: trigger on transition from not-pressed to pressed
                if (f9Current && !_f9PreviousState)
                {
                    WriteDebug("F9 pressed (transition detected)");
                    if (OnF9Pressed != null)
                        OnF9Pressed.Invoke();
                    else
                        DebugOverlayBehaviour.Instance?.Toggle();
                }
                _f9PreviousState = f9Current;

                // F10: trigger on transition from not-pressed to pressed
                if (f10Current && !_f10PreviousState)
                {
                    WriteDebug("F10 pressed (transition detected)");
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
                WriteDebug($"KeyInputSystem.OnUpdate EXCEPTION: {ex.GetType().Name}: {ex.Message}\n{ex.StackTrace}");
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
                    WriteDebug("[KeyInputSystem.RecreateInCurrentWorld] DefaultWorld null/disposed — scanning all worlds...");
                    foreach (World w in World.All)
                    {
                        if (w.IsCreated)
                        {
                            current = w;
                            WriteDebug($"[KeyInputSystem.RecreateInCurrentWorld] Found valid world: '{w.Name}'.");
                            break;
                        }
                    }
                }
                if (current == null || !current.IsCreated)
                {
                    WriteDebug("[KeyInputSystem.RecreateInCurrentWorld] No valid world found.");
                    return;
                }
                WriteDebug($"[KeyInputSystem.RecreateInCurrentWorld] Calling GetOrCreateSystem in '{current.Name}' (IsCreated={current.IsCreated}).");
                KeyInputSystem sys = current.GetOrCreateSystem<KeyInputSystem>();
                WriteDebug($"[KeyInputSystem.RecreateInCurrentWorld] Got system: World={sys.World?.Name ?? "null"} IsCreated={sys.World?.IsCreated ?? false}");
                // Update the cached world so GetActiveWorld() returns the current world.
                _cachedWorld = current;
                WriteDebug($"[KeyInputSystem.RecreateInCurrentWorld] Registered in '{current.Name}'.");
            }
            catch (Exception ex)
            {
                WriteDebug($"[KeyInputSystem.RecreateInCurrentWorld] Failed: {ex.Message}\n{ex.StackTrace}");
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
                WriteDebug($"EnsureOverlay: attached DebugOverlayBehaviour to camera '{cam.name}'");
            }
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = System.IO.Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                System.IO.File.AppendAllText(debugLog, $"[{System.DateTime.Now}] [{nameof(KeyInputSystem)}] {msg}\n");
            }
            catch { }
        }
    }
}
