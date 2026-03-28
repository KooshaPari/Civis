#nullable enable
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
        /// <summary>Called when F9 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF9Pressed;

        /// <summary>Called when F10 is pressed (set by RuntimeDriver if alive).</summary>
        public static System.Action? OnF10Pressed;

        /// <summary>Called when pack reload is requested (set by RuntimeDriver if alive). Can be invoked from background thread.</summary>
        public static System.Action? OnPackReloadRequested;

        private bool _overlayEnsured;
        private int _updateFrame;
        private bool _f9PreviousState;
        private bool _f10PreviousState;

        protected override void OnCreate()
        {
            WriteDebug("KeyInputSystem.OnCreate");
            Enabled = true;
            WriteDebug($"KeyInputSystem.OnCreate complete, Enabled={Enabled}");
        }

        protected override void OnDestroy()
        {
            WriteDebug("KeyInputSystem.OnDestroy called");
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

                // PlayerLoop drain injection has been removed in this version.

                // If PersistentRoot was destroyed by DINO, resurrect it via ECS
                Plugin.TryResurrect("(ECS tick)", "KeyInputSystem");

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
                bool f9Current  = Input.GetKey(KeyCode.F9);
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
            }
            catch (System.Exception ex)
            {
                WriteDebug($"KeyInputSystem.OnUpdate EXCEPTION: {ex.GetType().Name}: {ex.Message}\n{ex.StackTrace}");
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
