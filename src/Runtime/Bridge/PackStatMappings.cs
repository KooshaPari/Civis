#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.SDK.Models;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Pure-C# lookup tables and stat-path resolution logic for <see cref="PackStatInjector"/>.
    /// Contains no Unity.Entities or BepInEx dependencies so it compiles in CI without the
    /// game installed and is directly unit-testable from the SDK-only test project.
    ///
    /// This is the data layer; <see cref="PackStatInjector"/> is the ECS execution layer.
    /// </summary>
    public static class PackStatMappings
    {
        /// <summary>
        /// Maps pack <c>vanilla_mapping</c> strings to their primary ECS component type name.
        /// A null value signals an intentional skip (the entry is present so callers can
        /// distinguish "unknown mapping" from "known-but-skipped mapping such as aerial_fighter").
        /// </summary>
        public static readonly IReadOnlyDictionary<string, string?> VanillaMappingToComponentType =
            new Dictionary<string, string?>(StringComparer.OrdinalIgnoreCase)
            {
                // Melee archetypes
                { "militia",         "Components.MeleeUnit" },
                { "line_infantry",   "Components.MeleeUnit" },
                { "heavy_infantry",  "Components.MeleeUnit" },
                { "elite",           "Components.MeleeUnit" },
                { "hero",            "Components.MeleeUnit" },
                { "wall_defender",   "Components.MeleeUnit" },
                { "support",         "Components.MeleeUnit" },
                { "special",         "Components.MeleeUnit" },

                // Ranged archetypes
                { "ranged_infantry", "Components.RangeUnit" },
                { "skirmisher",      "Components.RangeUnit" },
                { "scout",           "Components.RangeUnit" },

                // Other archetypes
                { "cavalry",         "Components.CavalryUnit" },
                { "siege",           "Components.SiegeUnit" },

                // Intentionally null for stat injection: PackStatInjector skips aerial_fighter
                // (AerialSpawnSystem owns aerial behaviour). NOTE: the visual mesh swap targets
                // aerial units separately — see AssetSwapSystem.ResolveAerialArchetypeTypeName /
                // VanillaMappingToMeshSubstrings (BUG B fix #101). Do NOT assume null here means
                // "no swap target".
                { "aerial_fighter",  null },

                // ----------------------------------------------------------------------
                // #975 Phase 1 — full-world conversion: cims (citizens/workers) + buildings.
                //
                // CIMS: DINO's roaming non-combatant population ("cims"). CONFIRMED via live
                // entity dump (#986, build 1BDC999C, 2026-05-31): the renderable cim entity
                // carries Components.Citizen and RenderMesh ON THE SAME ENTITY (count=58 in a
                // live skirmish). The previous guess Components.Worker matched only 2 entities
                // (worker singletons), so the RenderMesh+Worker query returned 0 → "0 succeeded".
                // Components.Citizen is the correct archetype.
                { "cims",            "Components.Citizen" },
                { "worker",          "Components.Citizen" },   // alias for cims
                { "citizen",         "Components.Citizen" },   // alias for cims

                // BUILDINGS: every DINO building carries the Components.BuildingBase zero-size
                // marker (confirmed via entity crosswalk). Buildings register with a
                // vanilla_mapping resolved from building.vanilla_mapping → building_type →
                // "building" (see ContentLoader.RegisterAssetSwaps, Gap A). All building-type
                // values map to BuildingBase; mesh-name substrings (AssetSwapSystem) provide the
                // optional secondary refinement so distinct building meshes can be swapped
                // selectively as their vanilla mesh names become known.
                { "building",        "Components.BuildingBase" },
                { "command",         "Components.BuildingBase" },
                { "barracks",        "Components.BuildingBase" },
                { "resource",        "Components.BuildingBase" },
                { "economy",         "Components.BuildingBase" },
                { "defense",         "Components.BuildingBase" },
                { "tower",           "Components.BuildingBase" },
                { "wall",            "Components.BuildingBase" },
                { "research",        "Components.BuildingBase" },
            };

        /// <summary>
        /// Resolves a <c>vanilla_mapping</c> string to its ECS component type name.
        /// </summary>
        /// <param name="vanillaMapping">The <c>vanilla_mapping</c> value from a pack unit definition.</param>
        /// <param name="componentType">
        /// The resolved ECS component type name, or null if the mapping is intentionally skipped.
        /// Undefined when the method returns false.
        /// </param>
        /// <returns>
        /// True if the mapping string is registered (component type may still be null for
        /// intentionally skipped entries). False if the string is unrecognised or blank.
        /// </returns>
        public static bool TryResolveMapping(string? vanillaMapping, out string? componentType)
        {
            componentType = null;
            if (string.IsNullOrWhiteSpace(vanillaMapping))
                return false;

            return VanillaMappingToComponentType.TryGetValue(vanillaMapping!, out componentType);
        }

        /// <summary>
        /// Enumerates (sdkModelPath, floatValue) pairs for each non-zero stat in
        /// <paramref name="stats"/> that has a confirmed entry in <see cref="ComponentMap"/>.
        ///
        /// Confirmed SDK paths:
        ///   hp        → "unit.stats.hp"              (Components.Health / currentHealth)
        ///   armor     → "unit.stats.armor"            (Components.ArmorData / type)
        ///   speed     → "unit.stats.speed"            (Components.RawComponents.MoveHeading / speed)
        ///   fire_rate → "unit.stats.attack_cooldown"  (Components.AttackCooldown / value)
        ///   range     → "unit.stats.range"            (Components.GroundAttackArea)
        /// </summary>
        public static IEnumerable<(string SdkPath, float Value)> EnumerateStatPaths(UnitStats stats)
        {
            if (stats == null) throw new ArgumentNullException(nameof(stats));

            if (stats.Hp > 0f)
                yield return ("unit.stats.hp", stats.Hp);

            if (stats.Armor > 0f)
                yield return ("unit.stats.armor", stats.Armor);

            if (stats.Speed > 0f)
                yield return ("unit.stats.speed", stats.Speed);

            if (stats.FireRate > 0f)
                yield return ("unit.stats.attack_cooldown", stats.FireRate);

            if (stats.Range > 0f)
                yield return ("unit.stats.range", stats.Range);
        }
    }
}
