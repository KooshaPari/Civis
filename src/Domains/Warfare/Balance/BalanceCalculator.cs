using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Warfare.Archetypes;
using DINOForge.Domains.Warfare.Doctrines;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Warfare.Balance
{
    /// <summary>
    /// Calculates power ratings for units and factions, and compares factions for balance.
    /// Power rating is a composite score derived from a unit's effective stats after
    /// archetype and doctrine modifiers.
    /// </summary>
    public sealed class BalanceCalculator
    {
        private readonly DoctrineEngine _doctrineEngine;

        /// <summary>
        /// Creates a new balance calculator.
        /// </summary>
        /// <param name="doctrineEngine">The doctrine engine used to compute effective stats.</param>
        public BalanceCalculator(DoctrineEngine doctrineEngine)
        {
            _doctrineEngine = doctrineEngine ?? throw new ArgumentNullException(nameof(doctrineEngine));
        }

        /// <summary>
        /// Calculate effective power rating for a single unit, considering archetype and doctrine modifiers.
        /// Power = (HP * Armor_factor) * (Damage * FireRate * Accuracy) * Speed_factor / Cost_factor.
        /// </summary>
        /// <param name="unit">The unit definition.</param>
        /// <param name="archetype">The faction archetype.</param>
        /// <param name="doctrine">Optional doctrine.</param>
        /// <returns>A composite power rating score.</returns>
        public float CalculatePowerRating(UnitDefinition unit, FactionArchetype archetype, DoctrineDefinition? doctrine)
        {
            if (unit == null) throw new ArgumentNullException(nameof(unit));
            if (archetype == null) throw new ArgumentNullException(nameof(archetype));

            UnitStats effective = _doctrineEngine.ApplyAll(unit.Stats, archetype, doctrine);
            return ComputePower(effective);
        }

        /// <summary>
        /// Calculate overall faction power from all units belonging to the faction.
        /// </summary>
        /// <param name="faction">The faction definition.</param>
        /// <param name="units">The unit registry.</param>
        /// <param name="archetype">The faction archetype.</param>
        /// <param name="doctrine">Optional doctrine.</param>
        /// <returns>A faction power report with aggregate and per-unit ratings.</returns>
        public FactionPowerReport CalculateFactionPower(
            FactionDefinition faction,
            IRegistry<UnitDefinition> units,
            FactionArchetype archetype,
            DoctrineDefinition? doctrine)
        {
            if (faction == null) throw new ArgumentNullException(nameof(faction));
            if (units == null) throw new ArgumentNullException(nameof(units));
            if (archetype == null) throw new ArgumentNullException(nameof(archetype));

            string factionId = faction.Faction.Id;
            var unitPowerRatings = new Dictionary<string, float>(StringComparer.OrdinalIgnoreCase);
            float totalPower = 0f;

            foreach (KeyValuePair<string, RegistryEntry<UnitDefinition>> kvp in units.All)
            {
                UnitDefinition unit = kvp.Value.Data;
                if (!string.Equals(unit.FactionId, factionId, StringComparison.OrdinalIgnoreCase))
                    continue;

                float power = CalculatePowerRating(unit, archetype, doctrine);
                unitPowerRatings[unit.Id] = power;
                totalPower = totalPower + power;
            }

            int count = unitPowerRatings.Count;
            float avgPower = count > 0 ? totalPower / count : 0f;

            return new FactionPowerReport(
                factionId: factionId,
                archetypeId: archetype.Id,
                doctrineId: doctrine?.Id,
                totalPower: totalPower,
                averagePower: avgPower,
                unitCount: count,
                unitPowerRatings: unitPowerRatings);
        }

        /// <summary>
        /// Compare two faction power reports and produce a balance assessment.
        /// </summary>
        /// <param name="faction1">First faction's power report.</param>
        /// <param name="faction2">Second faction's power report.</param>
        /// <returns>A comparison report with delta, ratio, and assessment.</returns>
        public BalanceComparisonReport CompareFactions(
            FactionPowerReport faction1,
            FactionPowerReport faction2)
        {
            if (faction1 == null) throw new ArgumentNullException(nameof(faction1));
            if (faction2 == null) throw new ArgumentNullException(nameof(faction2));

            float delta = faction1.TotalPower - faction2.TotalPower;
            float ratio = faction2.TotalPower > 0f
                ? faction1.TotalPower / faction2.TotalPower
                : (faction1.TotalPower > 0f ? float.MaxValue : 1.0f);

            string assessment;
            string? stronger;

            float absDelta = Math.Abs(ratio - 1.0f);
            if (absDelta < 0.10f)
            {
                assessment = "balanced";
                stronger = null;
            }
            else if (absDelta < 0.25f)
            {
                assessment = "slight_advantage";
                stronger = delta > 0 ? faction1.FactionId : faction2.FactionId;
            }
            else
            {
                assessment = "significant_advantage";
                stronger = delta > 0 ? faction1.FactionId : faction2.FactionId;
            }

            return new BalanceComparisonReport(
                faction1: faction1,
                faction2: faction2,
                powerDelta: delta,
                powerRatio: ratio,
                assessment: assessment,
                strongerFaction: stronger);
        }

        /// <summary>
        /// Compute a raw power score from effective stats.
        /// Formula: (HP * (1 + Armor/100)) * (Damage * FireRate * Accuracy) * (1 + Speed/10) / max(TotalCost, 1)
        /// </summary>
        private static float ComputePower(UnitStats stats)
        {
            float survivability = stats.Hp * (1f + stats.Armor / 100f);
            float dps = stats.Damage * stats.FireRate * stats.Accuracy;
            float mobility = 1f + stats.Speed / 10f;
            int totalCost = stats.Cost.Food + stats.Cost.Wood + stats.Cost.Stone +
                            stats.Cost.Iron + stats.Cost.Gold + stats.Cost.Population;
            float costFactor = Math.Max(totalCost, 1);

            return (survivability * dps * mobility) / costFactor;
        }
    }
}
