using System;
using System.IO;
using BepInEx;
using DINOForge.Runtime.Diagnostics;
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
    [BepInPlugin("com.dinoforge.aviation", "DINOForge Aviation", PluginInfo.BEPINEX_VERSION)]
    [BepInDependency("com.dinoforge.runtime", BepInDependency.DependencyFlags.HardDependency)]
    public class AviationPlugin : BaseUnityPlugin
    {
        private void Awake()
        {
            DebugLog.Write("AviationPlugin", "AviationPlugin.Awake: Aviation subsystem loaded");
            DebugLog.Write("AviationPlugin", "  AerialUnitComponent, AntiAirComponent registered via assembly scan");
            DebugLog.Write("AviationPlugin", "  AerialMovementSystem, AerialSpawnSystem, AerialTargetingSystem will auto-register with SimulationSystemGroup");
        }
    }
}
