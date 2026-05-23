using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.Domains.Economy.Trade;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Economy.Balance
{
    /// <summary>
    /// Generates economy balance reports by analyzing faction production, consumption,
    /// and trade capabilities from the registries. Used for pack validation and balance tuning.
    /// </summary>
    public sealed class EconomyBalanceCalculator
    {
        private readonly ProductionCalculator _productionCalculator;
        private readonly TradeEngine _tradeEngine;

        /// <summary>
        /// Creates a new economy balance calculator with the given subsystem dependencies.
        /// </summary>
        /// <param name="productionCalculator">Calculator for building production rates.</param>
        /// <param name="tradeEngine">Engine for trade route evaluation.</param>
        public EconomyBalanceCalculator(ProductionCalculator productionCalculator, TradeEngine tradeEngine)
        {
            _productionCalculator = productionCalculator ?? throw new ArgumentNullException(nameof(productionCalculator));
            _tradeEngine = tradeEngine ?? throw new ArgumentNullException(nameof(tradeEngine));
        }

        /// <summary>
        /// Generates a comprehensive economy balance report for all factions in a given pack.
        /// Analyzes production rates, consumption, trade efficiency, and sustainability.
        /// </summary>
        /// <param name="packId">The pack identifier to scope the analysis to.</param>
        /// <param name="registries">The registry manager with all loaded content.</param>
        /// <param name="profiles">Economy profiles keyed by faction ID.</param>
        /// <param name="tradeRoutes">Available trade routes for the pack.</param>
        /// <returns>A complete economy balance report.</returns>
        public EconomyBalanceReport GenerateReport(
            string packId,
            RegistryManager registries,
            IReadOnlyDictionary<string, EconomyProfile> profiles,
            IReadOnlyList<TradeRoute> tradeRoutes)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (registries == null) throw new ArgumentNullException(nameof(registries));
            if (profiles == null) throw new ArgumentNullException(nameof(profiles));
            if (tradeRoutes == null) throw new ArgumentNullException(nameof(tradeRoutes));

            Dictionary<string, FactionEconomySummary> summaries = new Dictionary<string, FactionEconomySummary>(StringComparer.OrdinalIgnoreCase);
            List<string> warnings = new List<string>();

            foreach (KeyValuePair<string, RegistryEntry<FactionDefinition>> kvp in registries.Factions.All)
            {
                RegistryEntry<FactionDefinition> entry = kvp.Value;
                if (!string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase))
                    continue;

                FactionDefinition faction = entry.Data;
                string factionId = faction.Faction.Id;

                // Get or create a default profile for this faction
                EconomyProfile profile = profiles.TryGetValue(factionId, out EconomyProfile? fp) && fp != null
                    ? fp
                    : new EconomyProfile();

                // Gather faction building IDs
                List<string> buildingIds = GetFactionBuildingIds(faction);

                // Calculate production from buildings
                Dictionary<string, float> production = _productionCalculator.CalculateFactionProduction(
                    factionId, profile, registries.Buildings, buildingIds);

                // Calculate consumption (simplified: use building costs as proxy for upkeep)
                Dictionary<string, float> consumption = CalculateSimplifiedConsumption(faction, profile);

                // Calculate net balance (note: production, consumption, netBalance all have StringComparer.Ordinal from their constructors)
                Dictionary<string, float> netBalance = _productionCalculator.GetResourceBalance(production, consumption);

                // Count surpluses and deficits
                int surplusCount = netBalance.Values.Count(v => v > 0);
                int deficitCount = netBalance.Values.Count(v => v < 0);

                // Calculate sustainability score
                float sustainabilityScore = CalculateSustainabilityScore(netBalance);

                // Calculate trade efficiency
                float tradeEfficiency = CalculateTradeEfficiency(
                    tradeRoutes, profile, netBalance);

                // Generate warnings
                foreach (string resourceType in ResourceRate.ValidResourceTypes)
                {
                    float prodRate = production.TryGetValue(resourceType, out float p) ? p : 0f;
                    float balance = netBalance.TryGetValue(resourceType, out float b) ? b : 0f;

                    if (prodRate <= 0)
                    {
                        warnings.Add($"Faction '{factionId}' has no {resourceType} production.");
                    }
                    else if (balance < 0)
                    {
                        warnings.Add($"Faction '{factionId}' has a {resourceType} deficit of {Math.Abs(balance):F1}/tick.");
                    }
                }

                FactionEconomySummary summary = new FactionEconomySummary(
                    factionId: factionId,
                    production: production,
                    consumption: consumption,
                    netBalance: netBalance,
                    tradeEfficiency: tradeEfficiency,
                    sustainabilityScore: sustainabilityScore,
                    deficitCount: deficitCount,
                    surplusCount: surplusCount);

                summaries[factionId] = summary;
            }

            // Calculate overall balance score
            float overallBalance = CalculateOverallBalance(summaries.Values.ToList());

            return new EconomyBalanceReport(
                packId: packId,
                factionSummaries: summaries,
                overallBalanceScore: overallBalance,
                warnings: warnings.AsReadOnly());
        }

        private static List<string> GetFactionBuildingIds(FactionDefinition faction)
        {
            List<string> ids = new List<string>();
            FactionBuildings buildings = faction.Buildings;

            if (!string.IsNullOrEmpty(buildings.EconomyPrimary)) ids.Add(buildings.EconomyPrimary!);
            if (!string.IsNullOrEmpty(buildings.EconomySecondary)) ids.Add(buildings.EconomySecondary!);
            if (!string.IsNullOrEmpty(buildings.Barracks)) ids.Add(buildings.Barracks!);
            if (!string.IsNullOrEmpty(buildings.Workshop)) ids.Add(buildings.Workshop!);
            if (!string.IsNullOrEmpty(buildings.ArtilleryFoundry)) ids.Add(buildings.ArtilleryFoundry!);
            if (!string.IsNullOrEmpty(buildings.CommandCenter)) ids.Add(buildings.CommandCenter!);
            if (!string.IsNullOrEmpty(buildings.ResearchFacility)) ids.Add(buildings.ResearchFacility!);

            return ids;
        }

        private static Dictionary<string, float> CalculateSimplifiedConsumption(
            FactionDefinition faction,
            EconomyProfile profile)
        {
            Dictionary<string, float> consumption = new Dictionary<string, float>(StringComparer.Ordinal);

            // Base consumption: food is always consumed (population upkeep)
            float foodConsumption = 5.0f * profile.GetConsumptionMultiplier("food");
            consumption["food"] = foodConsumption;

            // Wood and stone consumed for building maintenance
            consumption["wood"] = 1.0f * profile.GetConsumptionMultiplier("wood");
            consumption["stone"] = 0.5f * profile.GetConsumptionMultiplier("stone");

            // Iron consumed for military upkeep based on army modifiers
            float ironConsumption = 2.0f * faction.Economy.UpkeepModifier * profile.GetConsumptionMultiplier("iron");
            consumption["iron"] = ironConsumption;

            // Gold consumed minimally (used for high-tier purchases)
            consumption["gold"] = 0.5f * profile.GetConsumptionMultiplier("gold");

            return consumption;
        }

        /// <summary>
        /// Calculates how self-sustaining a faction is based on its net resource balance.
        /// Score of 1.0 means all resources are in surplus; 0.0 means all are in deficit.
        /// </summary>
        private static float CalculateSustainabilityScore(Dictionary<string, float> netBalance)
        {
            if (netBalance.Count == 0) return 0f;

            int positiveCount = netBalance.Values.Count(v => v >= 0);
            return (float)positiveCount / netBalance.Count;
        }

        /// <summary>
        /// Calculates trade efficiency by checking if available trade routes can cover deficits.
        /// </summary>
        private float CalculateTradeEfficiency(
            IReadOnlyList<TradeRoute> routes,
            EconomyProfile profile,
            Dictionary<string, float> netBalance)
        {
            List<string> deficits = netBalance
                .Where(kvp => kvp.Value < 0)
                .Select(kvp => kvp.Key)
                .ToList();

            if (deficits.Count == 0) return 1.0f; // No deficits = perfect trade position

            int coverableDeficits = 0;
            float totalEfficiency = 0f;

            foreach (string deficit in deficits)
            {
                // Find best route that produces the deficit resource
                float bestEfficiency = 0f;
                foreach (TradeRoute route in routes)
                {
                    if (!route.Enabled) continue;
                    if (!string.Equals(route.TargetResource, deficit, StringComparison.OrdinalIgnoreCase))
                        continue;

                    float effectiveRate = _tradeEngine.CalculateExchangeRate(route, profile);
                    float efficiency = effectiveRate > 0 ? Math.Min(1.0f / effectiveRate, 1.0f) : 0f;

                    // Check if the source resource is in surplus
                    float sourceBalance = netBalance.TryGetValue(route.SourceResource, out float sb) ? sb : 0f;
                    if (sourceBalance > 0 && efficiency > bestEfficiency)
                    {
                        bestEfficiency = efficiency;
                    }
                }

                if (bestEfficiency > 0)
                {
                    coverableDeficits++;
                    totalEfficiency = totalEfficiency + bestEfficiency;
                }
            }

            if (deficits.Count == 0) return 1.0f;
            return totalEfficiency / deficits.Count;
        }

        /// <summary>
        /// Calculates how evenly balanced economies are across all factions.
        /// Compares sustainability scores - closer scores mean better balance.
        /// </summary>
        private static float CalculateOverallBalance(List<FactionEconomySummary> summaries)
        {
            if (summaries.Count <= 1) return 1.0f;

            List<float> scores = summaries.Select(s => s.SustainabilityScore).ToList();
            float average = scores.Average();

            if (average <= 0) return 0f;

            // Calculate coefficient of variation (lower = more balanced)
            float variance = scores.Sum(s => (s - average) * (s - average)) / scores.Count;
            float stdDev = (float)Math.Sqrt(variance);
            float coeffOfVariation = stdDev / average;

            // Convert to 0-1 score where 1.0 is perfectly balanced
            return Math.Max(0f, 1.0f - coeffOfVariation);
        }
    }
}
