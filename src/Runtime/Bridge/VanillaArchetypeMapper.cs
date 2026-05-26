using System;
using System.Collections.Generic;

namespace DINOForge.Runtime.Bridge
{
    /// <summary>
    /// Maps pack unit class identifiers to vanilla DINO ECS unit class component types.
    /// This is essential for unit spawning: when a pack defines a custom unit with class "melee",
    /// the spawner needs to know which vanilla archetype entity to clone as the template.
    ///
    /// Mapping strategy:
    ///   - Pack unit classes (from UnitDefinition.UnitClass) are matched to vanilla archetypes
    ///   - Vanilla archetypes are identified by their ECS component type tags
    ///   - A sample vanilla entity of that class is found and cloned to create the pack unit
    ///   - The clone is then configured with pack-specific stats via StatModifierSystem
    ///
    /// Valid pack unit classes:
    ///   MilitiaLight, CoreLineInfantry, EliteLineInfantry, HeavyInfantry, Skirmisher,
    ///   AntiArmor, ShockMelee, SwarmFodder, FastVehicle, MainBattleVehicle, HeavySiege,
    ///   Artillery, WalkerHeavy, StaticMG, StaticAT, StaticArtillery, SupportEngineer,
    ///   Recon, HeroCommander, AirstrikeProxy, ShieldedElite.
    /// </summary>
    public static class VanillaArchetypeMapper
    {
        /// <summary>
        /// Maps unit class names to the ECS component type that identifies vanilla archetypes
        /// of that class.
        /// </summary>
        private static readonly Dictionary<string, string> ClassToComponentType =
            new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase)
            {
                // Melee classes
                { "MilitiaLight", "Components.MeleeUnit" },
                { "CoreLineInfantry", "Components.MeleeUnit" },
                { "EliteLineInfantry", "Components.MeleeUnit" },
                { "HeavyInfantry", "Components.MeleeUnit" },
                { "ShockMelee", "Components.MeleeUnit" },
                { "SwarmFodder", "Components.MeleeUnit" },

                // Ranged/Skirmisher classes
                { "Skirmisher", "Components.RangeUnit" },
                { "AntiArmor", "Components.RangeUnit" },

                // Cavalry classes
                { "FastVehicle", "Components.CavalryUnit" },

                // Siege classes
                { "MainBattleVehicle", "Components.SiegeUnit" },
                { "HeavySiege", "Components.SiegeUnit" },
                { "Artillery", "Components.SiegeUnit" },
                { "WalkerHeavy", "Components.SiegeUnit" },

                // Archery classes
                { "Archer", "Components.Archer" },

                // Special/support classes (map to closest melee archetype as fallback)
                { "StaticMG", "Components.MeleeUnit" },
                { "StaticAT", "Components.MeleeUnit" },
                { "StaticArtillery", "Components.SiegeUnit" },
                { "SupportEngineer", "Components.MeleeUnit" },
                { "Recon", "Components.RangeUnit" },
                { "HeroCommander", "Components.MeleeUnit" },
                { "AirstrikeProxy", "Components.MeleeUnit" },
                { "ShieldedElite", "Components.MeleeUnit" },
            };

        /// <summary>
        /// Get the vanilla archetype component type for a pack unit class.
        /// </summary>
        /// <param name="unitClass">The unit class from UnitDefinition.UnitClass.</param>
        /// <returns>The ECS component type name (e.g. "Components.MeleeUnit"), or null if unmapped.</returns>
        public static string? MapUnitClassToComponentType(string unitClass)
        {
            if (string.IsNullOrWhiteSpace(unitClass))
                return null;

            if (ClassToComponentType.TryGetValue(unitClass, out string? componentType))
                return componentType;

            return null;
        }

        /// <summary>
        /// Check if a unit class can be spawned by looking up its mapping.
        /// </summary>
        /// <param name="unitClass">The unit class from UnitDefinition.UnitClass.</param>
        /// <returns>True if the class has a mapped archetype.</returns>
        public static bool IsSpawnable(string unitClass)
        {
            return MapUnitClassToComponentType(unitClass) != null;
        }

        /// <summary>
        /// Validate that a pack unit class is recognized, for use in pack validation.
        /// </summary>
        /// <param name="unitClass">The unit class to validate.</param>
        /// <param name="errorMessage">Output error message if invalid.</param>
        /// <returns>True if valid.</returns>
        public static bool ValidateUnitClass(string unitClass, out string? errorMessage)
        {
            errorMessage = null;

            if (string.IsNullOrWhiteSpace(unitClass))
            {
                errorMessage = "Unit class is empty or null";
                return false;
            }

            if (!ClassToComponentType.ContainsKey(unitClass))
            {
                errorMessage = $"Unknown unit class '{unitClass}'. " +
                    "Valid classes: MilitiaLight, CoreLineInfantry, EliteLineInfantry, HeavyInfantry, Skirmisher, " +
                    "AntiArmor, ShockMelee, SwarmFodder, FastVehicle, MainBattleVehicle, HeavySiege, " +
                    "Artillery, WalkerHeavy, StaticMG, StaticAT, StaticArtillery, SupportEngineer, " +
                    "Recon, HeroCommander, AirstrikeProxy, ShieldedElite.";
                return false;
            }

            return true;
        }
    }
}
