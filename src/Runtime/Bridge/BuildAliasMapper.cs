#nullable enable
using System;
using DINOForge.SDK.Models;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Pure (Unity-free, unit-testable) logic that resolves which vanilla DINO
    /// <c>Utility.EnumsStorage.BuildingType</c> a pack building should ALIAS into the live
    /// build menu.
    ///
    /// DINO's build menu is keyed by the <b>closed compiled <c>BuildingType</c> enum</b>
    /// (see docs/sessions/dino-build-catalog-20260530.md). A genuinely new building type
    /// cannot be added to the native menu at runtime, so every pack building must ride on an
    /// existing buildable slot. The runtime then reskins (mesh-swap, #964) and re-targets
    /// (UnitsShop production) that slot so the player sees the pack building.
    ///
    /// Resolution order (declarative-first, no hardcoded content IDs):
    ///   1. Explicit <c>build_alias:</c> on the building definition (validated against the
    ///      live enum by <see cref="BuildMenuInjector"/>).
    ///   2. Auto-map by <see cref="BuildingDefinition.BuildingType"/> functional category.
    /// </summary>
    public static class BuildAliasMapper
    {
        /// <summary>Default vanilla alias for production buildings (trains a special unit class).</summary>
        public const string DefaultProductionAlias = "Stables";

        /// <summary>Default vanilla alias for defensive buildings/towers.</summary>
        public const string DefaultDefenseAlias = "Tower";

        /// <summary>Default vanilla alias for naval / dock buildings.</summary>
        public const string DefaultNavalAlias = "Port";

        /// <summary>Fallback alias when no category matches.</summary>
        public const string FallbackAlias = "Stables";

        /// <summary>
        /// Resolves the vanilla <c>BuildingType</c> enum name a pack building aliases.
        /// Returns the explicit <c>build_alias</c> when present, else auto-maps from the
        /// building's functional category and id heuristics. Never returns null/empty.
        /// </summary>
        /// <param name="def">The pack building definition.</param>
        /// <returns>A vanilla <c>BuildingType</c> enum member name (e.g. "Stables", "Tower", "Port").</returns>
        public static string ResolveAlias(BuildingDefinition def)
        {
            if (def == null) throw new ArgumentNullException(nameof(def));

            if (!string.IsNullOrWhiteSpace(def.BuildAlias))
                return def.BuildAlias!.Trim();

            string category = (def.BuildingType ?? string.Empty).Trim();
            string id = (def.Id ?? string.Empty).ToLowerInvariant();

            // Naval heuristic: explicit naval category, or id/name hints.
            if (Matches(category, "naval", "port", "dock", "harbor", "harbour", "shipyard")
                || id.Contains("port") || id.Contains("dock") || id.Contains("ship")
                || id.Contains("naval") || id.Contains("harbor") || id.Contains("harbour"))
                return DefaultNavalAlias;

            // Defense heuristic.
            if (Matches(category, "defense", "defence", "tower", "turret", "anti_air", "antiair")
                || id.Contains("tower") || id.Contains("turret") || id.Contains("cannon")
                || (def.DefenseTags != null && def.DefenseTags.Contains("AntiAir")))
                return DefaultDefenseAlias;

            // Production heuristic (airfields, hangars, bays, barracks-like).
            if (Matches(category, "production", "barracks", "barraks", "training", "trainer", "factory")
                || id.Contains("bay") || id.Contains("hangar") || id.Contains("roost")
                || id.Contains("nest") || id.Contains("cave") || id.Contains("aviary")
                || id.Contains("airport") || id.Contains("airfield") || id.Contains("platform"))
                return DefaultProductionAlias;

            return FallbackAlias;
        }

        private static bool Matches(string value, params string[] needles)
        {
            if (string.IsNullOrEmpty(value)) return false;
            string v = value.ToLowerInvariant();
            foreach (string n in needles)
                if (v.IndexOf(n, StringComparison.Ordinal) >= 0)
                    return true;
            return false;
        }
    }
}
