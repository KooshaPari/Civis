#nullable enable
using DINOForge.Runtime.Diagnostics;
using Unity.Entities;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// One-shot ECS system that drives <see cref="BuildMenuInjector.RunInjection"/> on the
    /// Unity main thread once the game world is populated. Mirrors
    /// <c>AerialSpawnSystem</c>'s one-shot building sweep: it cannot run at the main menu (DINO
    /// construction/UI system groups only fire during active gameplay) and must run on the
    /// main thread because the injector touches the live config ScriptableObject via
    /// <c>Resources.FindObjectsOfTypeAll</c>.
    ///
    /// Uses <c>SystemBase.OnUpdate</c> (NOT MonoBehaviour.Update, which never fires in DINO).
    /// </summary>
    [UpdateInGroup(typeof(SimulationSystemGroup))]
    public class BuildMenuInjectionSystem : SystemBase
    {
        /// <summary>Frames to wait so the build config + world are fully loaded (~30s @ 60fps).</summary>
        private const int MinFrameDelay = 1800;

        private int _frameCount;
        private bool _injected;

        /// <inheritdoc />
        public override void OnCreate()
        {
            base.OnCreate();
            DebugLog.Write("BuildMenuInjector", "BuildMenuInjectionSystem.OnCreate");
        }

        /// <inheritdoc />
        public override void OnUpdate()
        {
            if (_injected) return;
            _frameCount++;
            if (_frameCount < MinFrameDelay) return;

            try
            {
                int n = BuildMenuInjector.RunInjection();
                DebugLog.Write("BuildMenuInjector",
                    $"BuildMenuInjectionSystem: injection pass complete, {n} pack building(s) registered.");
            }
            catch (System.Exception ex)
            {
                DebugLog.Write("BuildMenuInjector", $"BuildMenuInjectionSystem: injection failed: {ex.Message}");
            }
            finally
            {
                _injected = true;
            }
        }
    }
}
