using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.SDK.Models;

namespace DINOForge.Domains.Warfare.Doctrines
{
    /// <summary>
    /// Engine for applying doctrine and archetype modifiers to unit stats.
    /// All operations are pure: inputs are never mutated.
    /// </summary>
    public sealed class DoctrineEngine
    {
        /// <summary>
        /// Apply a doctrine's modifiers to a unit's base stats.
        /// Modifiers are treated as multipliers keyed by stat name.
        /// Returns a new <see cref="UnitStats"/> instance with modified values.
        /// </summary>
        /// <param name="baseStats">The unit's base stats (not mutated).</param>
        /// <param name="doctrine">The doctrine whose modifiers to apply.</param>
        /// <returns>A new UnitStats with doctrine modifiers applied.</returns>
        public UnitStats ApplyDoctrine(UnitStats baseStats, DoctrineDefinition doctrine)
        {
            if (baseStats == null) throw new ArgumentNullException(nameof(baseStats));
            if (doctrine == null) throw new ArgumentNullException(nameof(doctrine));

            return ApplyModifiers(baseStats, doctrine.Modifiers);
        }

        /// <summary>
        /// Apply both archetype and doctrine modifiers to a unit's base stats.
        /// Archetype modifiers are applied first, then doctrine modifiers (if any).
        /// Returns a new <see cref="UnitStats"/> instance.
        /// </summary>
        /// <param name="baseStats">The unit's base stats (not mutated).</param>
        /// <param name="archetype">The faction archetype to apply.</param>
        /// <param name="doctrine">Optional doctrine to apply after the archetype.</param>
        /// <returns>A new UnitStats with all modifiers applied.</returns>
        public UnitStats ApplyAll(UnitStats baseStats, FactionArchetype archetype, DoctrineDefinition? doctrine)
        {
            if (baseStats == null) throw new ArgumentNullException(nameof(baseStats));
            if (archetype == null) throw new ArgumentNullException(nameof(archetype));

            UnitStats result = ApplyModifiers(baseStats, archetype.BaseModifiers);

            if (doctrine != null)
            {
                result = ApplyModifiers(result, doctrine.Modifiers);
            }

            return result;
        }

        /// <summary>
        /// Validate that a doctrine's modifiers won't produce invalid stat values
        /// (e.g. negative HP, negative damage) when applied to reasonable base stats.
        /// </summary>
        /// <param name="doctrine">The doctrine to validate.</param>
        /// <returns>A list of validation error messages. Empty if valid.</returns>
        public IReadOnlyList<string> ValidateDoctrine(DoctrineDefinition doctrine)
        {
            if (doctrine == null) throw new ArgumentNullException(nameof(doctrine));

            var errors = new List<string>();

            foreach (KeyValuePair<string, float> kvp in doctrine.Modifiers)
            {
                if (kvp.Value < 0f)
                {
                    errors.Add($"Doctrine '{doctrine.Id}': modifier '{kvp.Key}' has negative value {kvp.Value}. " +
                               "Modifiers should be positive multipliers (e.g. 0.5 for -50%, 1.5 for +50%).");
                }

                if (kvp.Value > 10f)
                {
                    errors.Add($"Doctrine '{doctrine.Id}': modifier '{kvp.Key}' has extreme value {kvp.Value} (>10x). " +
                               "This may produce unbalanced results.");
                }
            }

            if (string.IsNullOrWhiteSpace(doctrine.Id))
            {
                errors.Add("Doctrine has no id.");
            }

            if (string.IsNullOrWhiteSpace(doctrine.DisplayName))
            {
                errors.Add("Doctrine has no display_name.");
            }

            return errors;
        }

        private static UnitStats ApplyModifiers(UnitStats stats, IReadOnlyDictionary<string, float> modifiers)
        {
            var result = new UnitStats
            {
                Hp = stats.Hp,
                Damage = stats.Damage,
                Armor = stats.Armor,
                Range = stats.Range,
                Speed = stats.Speed,
                Accuracy = stats.Accuracy,
                FireRate = stats.FireRate,
                Morale = stats.Morale,
                Cost = new ResourceCost
                {
                    Food = stats.Cost.Food,
                    Wood = stats.Cost.Wood,
                    Stone = stats.Cost.Stone,
                    Iron = stats.Cost.Iron,
                    Gold = stats.Cost.Gold,
                    Population = stats.Cost.Population
                }
            };

            foreach (KeyValuePair<string, float> kvp in modifiers)
            {
                ApplyModifier(result, kvp.Key, kvp.Value);
            }

            // Clamp to ensure no negative values
            result.Hp = Math.Max(result.Hp, 1f);
            result.Damage = Math.Max(result.Damage, 0f);
            result.Armor = Math.Max(result.Armor, 0f);
            result.Speed = Math.Max(result.Speed, 0f);
            result.Accuracy = Math.Max(Math.Min(result.Accuracy, 1f), 0f);
            result.FireRate = Math.Max(result.FireRate, 0.01f);
            result.Morale = Math.Max(result.Morale, 0f);

            return result;
        }

        private static void ApplyModifier(UnitStats stats, string key, float multiplier)
        {
            switch (key.ToLowerInvariant())
            {
                case "hp":
                case "health":
                    stats.Hp *= multiplier;
                    break;
                case "damage":
                    stats.Damage *= multiplier;
                    break;
                case "armor":
                case "defense":
                    stats.Armor *= multiplier;
                    break;
                case "range":
                    stats.Range *= multiplier;
                    break;
                case "speed":
                    stats.Speed *= multiplier;
                    break;
                case "accuracy":
                    stats.Accuracy *= multiplier;
                    break;
                case "fire_rate":
                case "firerate":
                    stats.FireRate *= multiplier;
                    break;
                case "morale":
                    stats.Morale *= multiplier;
                    break;
                case "cost":
                    stats.Cost.Food = (int)(stats.Cost.Food * multiplier);
                    stats.Cost.Wood = (int)(stats.Cost.Wood * multiplier);
                    stats.Cost.Stone = (int)(stats.Cost.Stone * multiplier);
                    stats.Cost.Iron = (int)(stats.Cost.Iron * multiplier);
                    stats.Cost.Gold = (int)(stats.Cost.Gold * multiplier);
                    break;
                    // Unknown modifiers are silently ignored (extensibility)
            }
        }
    }
}
