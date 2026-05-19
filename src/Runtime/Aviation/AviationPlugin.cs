using System;
using System.IO;
using BepInEx;
using Unity.Entities;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// BepInEx plugin entry point for the DINOForge Aviation subsystem.
    /// Registers ECS systems (AerialMovementSystem, AerialSpawnSystem) with the Unity ECS world.
    ///
    /// This plugin is automatically loaded by BepInEx from BepInEx/ecs_plugins/.
    /// The ECS component types (AerialUnitComponent, AntiAirComponent) are registered
    /// automatically by the Mono runtime when this assembly is loaded.
    ///
    /// No manual system registration is required — systems with [UpdateInGroup] attributes
    /// are auto-discovered by Unity ECS at world creation time.
    /// </summary>
    [BepInPlugin("com.dinoforge.aviation", "DINOForge Aviation", "0.1.0")]
    [BepInDependency("com.dinoforge.runtime", BepInDependency.DependencyFlags.HardDependency)]
    public class AviationPlugin : BaseUnityPlugin
    {
        private void Awake()
        {
            WriteDebug("AviationPlugin.Awake: Aviation subsystem loaded");
            WriteDebug("  AerialUnitComponent, AntiAirComponent registered via assembly scan");
            WriteDebug("  AerialMovementSystem, AerialSpawnSystem, AerialTargetingSystem will auto-register with SimulationSystemGroup");
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] {msg}\n");
            }
            catch (Exception ex)
            {
                /* safe-swallow: diagnostic log write failure is non-critical for aviation plugin */
                System.Diagnostics.Debug.WriteLine($"AviationPlugin debug log write failed: {ex.Message}");
            }
        }
    }
}
