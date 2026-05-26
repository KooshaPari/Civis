#nullable enable
using System;
using System.IO;
using DINOForge.Runtime.Diagnostics;
using DINOForge.SDK.Models;
using Unity.Entities;

namespace DINOForge.Runtime.Aviation
{
    /// <summary>
    /// Maps pack building YAML defense_tags to ECS components for anti-air buildings.
    ///
    /// Buildings are baked into the DINO game world at scene load time — there is no
    /// runtime building-spawner equivalent to PackUnitSpawner.  The correct call-site is
    /// therefore <see cref="AerialSpawnSystem"/>, which scans all existing building entities
    /// during its startup sweep and calls <see cref="ApplyAntiAirComponent"/> for each one
    /// whose DINOForge definition carries the "AntiAir" defense_tag.
    ///
    /// Supported defense_tags:
    ///   "AntiAir" → Attaches <see cref="AntiAirComponent"/> with parameters from the
    ///               <c>anti_air:</c> block (or defaults: range 20f, damage_bonus 1.5f).
    ///
    /// Usage (from AerialSpawnSystem or any future building placer):
    /// <code>
    ///   AerialBuildingMapper.ApplyAntiAirComponent(EntityManager, entity, buildingDef);
    /// </code>
    /// </summary>
    public static class AerialBuildingMapper
    {
        private const float DefaultAntiAirRange = 20f;
        private const float DefaultAntiAirDamageBonus = 1.5f;

        /// <summary>
        /// Inspects the building definition's defense tags and anti-air properties,
        /// and attaches <see cref="AntiAirComponent"/> to the entity when warranted.
        ///
        /// Safe to call multiple times on the same entity: a HasComponent guard prevents
        /// duplicate component additions.
        /// </summary>
        /// <param name="em">The EntityManager to use for component operations.</param>
        /// <param name="entity">The building entity (baked or spawned).</param>
        /// <param name="buildingDef">The pack building definition, deserialized from YAML.</param>
        public static void ApplyAntiAirComponent(EntityManager em, Entity entity, BuildingDefinition buildingDef)
        {
            if (buildingDef?.DefenseTags == null)
                return;

            if (!buildingDef.DefenseTags.Contains("AntiAir"))
                return;

            try
            {
                float range = DefaultAntiAirRange;
                float damageBonus = DefaultAntiAirDamageBonus;

                if (buildingDef.AntiAir != null)
                {
                    if (buildingDef.AntiAir.Range > 0f)
                        range = buildingDef.AntiAir.Range;

                    if (buildingDef.AntiAir.DamageBonus > 0f)
                        damageBonus = buildingDef.AntiAir.DamageBonus;
                }

                if (!em.HasComponent<AntiAirComponent>(entity))
                {
                    AntiAirComponent antiAir = new AntiAirComponent
                    {
                        AntiAirRange = range,
                        AntiAirDamageBonus = damageBonus
                    };
                    em.AddComponentData(entity, antiAir);
                    DebugLog.Write("AerialBuildingMapper", $"Applied AntiAirComponent to building '{buildingDef.Id}' (range={range}, damageBonus={damageBonus})");
                }
                else
                {
                    DebugLog.Write("AerialBuildingMapper", $"Building '{buildingDef.Id}' already has AntiAirComponent — skipped");
                }
            }
            catch (Exception ex)
            {
                DebugLog.Write("AerialBuildingMapper", $"Failed to add AntiAirComponent to building '{buildingDef.Id}': {ex.Message}");
            }
        }
    }
}
