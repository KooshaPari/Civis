using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Economy.Rates
{
    /// <summary>
    /// Calculates effective production rates for factions given their buildings,
    /// worker allocations, economy profiles, and pack modifiers.
    /// </summary>
    public sealed class ProductionCalculator
    {
        /// <summary>
        /// Calculates the total resource production for a faction across all its production buildings.
        /// Applies economy profile modifiers and worker efficiency to each building's output.
        /// </summary>
        /// <param name="factionId">The faction identifier to calculate production for.</param>
        /// <param name="profile">The economy profile providing faction-level modifiers.</param>
        /// <param name="buildings">The building registry containing all loaded building definitions.</param>
        /// <param name="buildingIds">The IDs of buildings owned by this faction.</param>
        /// <param name="workerCounts">Map of building ID to number of workers assigned. Null means default staffing (1 worker per building).</param>
        /// <returns>A dictionary of resource type to total effective production rate.</returns>
        public Dictionary<string, float> CalculateFactionProduction(
            string factionId,
            EconomyProfile profile,
            IRegistry<BuildingDefinition> buildings,
            IReadOnlyList<string> buildingIds,
            Dictionary<string, int>? workerCounts = null)
        {
            if (string.IsNullOrWhiteSpace(factionId))
                throw new ArgumentException("Faction ID is required.", nameof(factionId));
            if (profile == null) throw new ArgumentNullException(nameof(profile));
            if (buildings == null) throw new ArgumentNullException(nameof(buildings));
            if (buildingIds == null) throw new ArgumentNullException(nameof(buildingIds));

            Dictionary<string, float> totalProduction = new Dictionary<string, float>(StringComparer.Ordinal);

            foreach (string resourceType in ResourceRate.ValidResourceTypes)
            {
                totalProduction[resourceType] = 0f;
            }

            foreach (string buildingId in buildingIds)
            {
                BuildingDefinition? building = buildings.Get(buildingId);
                if (building == null) continue;

                Dictionary<string, float> buildingOutput = CalculateBuildingOutput(
                    building, profile, workerCounts);

                foreach (KeyValuePair<string, float> kvp in buildingOutput)
                {
                    if (totalProduction.ContainsKey(kvp.Key))
                    {
                        totalProduction[kvp.Key] += kvp.Value;
                    }
                    else
                    {
                        totalProduction[kvp.Key] = kvp.Value;
                    }
                }
            }

            return totalProduction;
        }

        /// <summary>
        /// Calculates the effective output of a single building, applying economy profile
        /// production multipliers and worker efficiency.
        /// </summary>
        /// <param name="building">The building definition to calculate output for.</param>
        /// <param name="profile">The economy profile providing modifiers.</param>
        /// <param name="workerCounts">Optional worker count overrides keyed by building ID.</param>
        /// <returns>A dictionary of resource type to effective production rate for this building.</returns>
        public Dictionary<string, float> CalculateBuildingOutput(
            BuildingDefinition building,
            EconomyProfile profile,
            Dictionary<string, int>? workerCounts = null)
        {
            if (building == null) throw new ArgumentNullException(nameof(building));
            if (profile == null) throw new ArgumentNullException(nameof(profile));

            Dictionary<string, float> output = new Dictionary<string, float>(StringComparer.Ordinal);

            int workers = 1;
            if (workerCounts != null && workerCounts.TryGetValue(building.Id, out int assignedWorkers))
            {
                workers = Math.Max(0, assignedWorkers);
            }

            float workerMultiplier = workers * profile.WorkerEfficiency;

            foreach (KeyValuePair<string, int> production in building.Production)
            {
                string resourceType = production.Key;
                float baseRate = production.Value;
                float profileMultiplier = profile.GetProductionMultiplier(resourceType);
                float effectiveRate = baseRate * profileMultiplier * workerMultiplier;
                output[resourceType] = effectiveRate;
            }

            return output;
        }

        /// <summary>
        /// Calculates the net resource balance for a faction: production minus consumption.
        /// Returns positive values for surplus resources and negative values for deficits.
        /// </summary>
        /// <param name="production">Per-resource production rates (from CalculateFactionProduction).</param>
        /// <param name="consumption">Per-resource consumption rates (unit upkeep, building maintenance, etc.).</param>
        /// <returns>A dictionary of resource type to net balance (production - consumption).</returns>
        public Dictionary<string, float> GetResourceBalance(
            Dictionary<string, float> production,
            Dictionary<string, float> consumption)
        {
            if (production == null) throw new ArgumentNullException(nameof(production));
            if (consumption == null) throw new ArgumentNullException(nameof(consumption));

            Dictionary<string, float> balance = new Dictionary<string, float>(StringComparer.Ordinal);

            // Start with all production
            foreach (KeyValuePair<string, float> kvp in production)
            {
                balance[kvp.Key] = kvp.Value;
            }

            // Subtract consumption
            foreach (KeyValuePair<string, float> kvp in consumption)
            {
                if (balance.ContainsKey(kvp.Key))
                {
                    balance[kvp.Key] -= kvp.Value;
                }
                else
                {
                    balance[kvp.Key] = -kvp.Value;
                }
            }

            return balance;
        }

        /// <summary>
        /// Calculates the total resource consumption from a set of units, given their upkeep costs
        /// and the economy profile's consumption modifiers.
        /// </summary>
        /// <param name="units">The unit registry containing all loaded unit definitions.</param>
        /// <param name="unitCounts">Map of unit ID to count of active units.</param>
        /// <param name="profile">The economy profile providing consumption modifiers.</param>
        /// <returns>A dictionary of resource type to total consumption rate.</returns>
        public Dictionary<string, float> CalculateUnitConsumption(
            IRegistry<UnitDefinition> units,
            Dictionary<string, int> unitCounts,
            EconomyProfile profile)
        {
            if (units == null) throw new ArgumentNullException(nameof(units));
            if (unitCounts == null) throw new ArgumentNullException(nameof(unitCounts));
            if (profile == null) throw new ArgumentNullException(nameof(profile));

            Dictionary<string, float> totalConsumption = new Dictionary<string, float>(StringComparer.Ordinal);

            foreach (KeyValuePair<string, int> kvp in unitCounts)
            {
                UnitDefinition? unit = units.Get(kvp.Key);
                if (unit == null) continue;

                int count = kvp.Value;
                ResourceCost cost = unit.Stats.Cost;

                AddConsumption(totalConsumption, "food", cost.Food * count, profile);
                AddConsumption(totalConsumption, "wood", cost.Wood * count, profile);
                AddConsumption(totalConsumption, "stone", cost.Stone * count, profile);
                AddConsumption(totalConsumption, "iron", cost.Iron * count, profile);
                AddConsumption(totalConsumption, "gold", cost.Gold * count, profile);
            }

            return totalConsumption;
        }

        private static void AddConsumption(
            Dictionary<string, float> totals,
            string resourceType,
            float amount,
            EconomyProfile profile)
        {
            if (amount <= 0) return;

            float modified = amount * profile.GetConsumptionMultiplier(resourceType);

            if (totals.ContainsKey(resourceType))
            {
                totals[resourceType] += modified;
            }
            else
            {
                totals[resourceType] = modified;
            }
        }
    }
}
