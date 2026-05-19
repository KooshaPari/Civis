using System;
using System.IO;
using DINOForge.SDK.Models;
using Unity.Entities;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// Maps pack unit YAML behavior_tags to ECS components for aerial units.
    /// Called by PackUnitSpawner after a unit entity is instantiated.
    ///
    /// Supported behavior_tags:
    ///   "Aerial"   → Attaches <see cref="AerialUnitComponent"/> with parameters from aerial block
    ///   "AntiAir"  → Attaches <see cref="AntiAirComponent"/> with default range/bonus
    ///
    /// Usage:
    ///   AerialUnitMapper.ApplyAerialComponents(EntityManager, spawned, unitDef);
    /// </summary>
    public static class AerialUnitMapper
    {
        private const float DefaultAntiAirRange = 20f;
        private const float DefaultAntiAirDamageBonus = 1.5f;

        /// <summary>
        /// Inspects the unit definition's behavior tags and aerial properties,
        /// and attaches the appropriate ECS components to the spawned entity.
        /// </summary>
        /// <param name="em">The EntityManager to use for component operations.</param>
        /// <param name="entity">The newly spawned entity.</param>
        /// <param name="unitDef">The pack unit definition.</param>
        public static void ApplyAerialComponents(EntityManager em, Entity entity, UnitDefinition unitDef)
        {
            if (unitDef?.BehaviorTags == null)
                return;

            bool isAerial = unitDef.BehaviorTags.Contains("Aerial");
            bool isAntiAir = unitDef.BehaviorTags.Contains("AntiAir");

            if (isAerial)
            {
                try
                {
                    AerialUnitComponent aerialComp = BuildAerialComponent(unitDef);
                    if (!em.HasComponent<AerialUnitComponent>(entity))
                    {
                        em.AddComponentData(entity, aerialComp);
                        WriteDebug($"Applied AerialUnitComponent to {unitDef.Id} (altitude={aerialComp.CruiseAltitude})");
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug($"Failed to add AerialUnitComponent to {unitDef.Id}: {ex.Message}");
                }
            }

            if (isAntiAir)
            {
                try
                {
                    AntiAirComponent antiAir = new AntiAirComponent
                    {
                        AntiAirRange = DefaultAntiAirRange,
                        AntiAirDamageBonus = DefaultAntiAirDamageBonus
                    };
                    if (!em.HasComponent<AntiAirComponent>(entity))
                    {
                        em.AddComponentData(entity, antiAir);
                        WriteDebug($"Applied AntiAirComponent to {unitDef.Id}");
                    }
                }
                catch (Exception ex)
                {
                    WriteDebug($"Failed to add AntiAirComponent to {unitDef.Id}: {ex.Message}");
                }
            }
        }

        private static AerialUnitComponent BuildAerialComponent(UnitDefinition unitDef)
        {
            float cruiseAltitude = 15f;
            float ascendSpeed = 5f;
            float descendSpeed = 3f;

            if (unitDef.Aerial != null)
            {
                cruiseAltitude = unitDef.Aerial.CruiseAltitude > 0f ? unitDef.Aerial.CruiseAltitude : cruiseAltitude;
                ascendSpeed = unitDef.Aerial.AscendSpeed > 0f ? unitDef.Aerial.AscendSpeed : ascendSpeed;
                descendSpeed = unitDef.Aerial.DescendSpeed > 0f ? unitDef.Aerial.DescendSpeed : descendSpeed;
            }

            return new AerialUnitComponent
            {
                CruiseAltitude = cruiseAltitude,
                AscendSpeed = ascendSpeed,
                DescendSpeed = descendSpeed,
                IsAttacking = false
            };
        }

        private static void WriteDebug(string msg)
        {
            try
            {
                string debugLog = System.IO.Path.Combine(
                    BepInEx.Paths.BepInExRootPath, "dinoforge_debug.log");
                File.AppendAllText(debugLog, $"[{DateTime.UtcNow:o}] AerialUnitMapper: {msg}\n");
            }
            catch (Exception ex)
            {
                System.Diagnostics.Debug.WriteLine($"AerialUnitMapper debug log write failed: {ex.Message}");
            }
        }
    }
}
