using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Economy.Validation
{
    /// <summary>
    /// Validates economy packs for correctness and consistency. Checks that:
    /// - All production buildings reference valid resource types
    /// - Trade routes have valid source and target resources
    /// - Economy profiles have non-negative starting resources
    /// - No circular trade dependencies exist
    /// - Exchange rates are within reasonable bounds
    /// </summary>
    public sealed class EconomyValidator
    {
        private static readonly HashSet<string> ValidResources = new HashSet<string>(
            ResourceRate.ValidResourceTypes, StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// Validates all economy-related content in the given registries and economy data.
        /// </summary>
        /// <param name="packId">The pack identifier to scope validation to.</param>
        /// <param name="registries">The registry manager with all loaded content.</param>
        /// <param name="profiles">Economy profiles defined in this pack.</param>
        /// <param name="tradeRoutes">Trade routes defined in this pack.</param>
        /// <returns>A validation result with all errors and warnings found.</returns>
        public EconomyValidationResult Validate(
            string packId,
            RegistryManager registries,
            IReadOnlyList<EconomyProfile> profiles,
            IReadOnlyList<TradeRoute> tradeRoutes)
        {
            if (string.IsNullOrWhiteSpace(packId))
                throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (registries == null) throw new ArgumentNullException(nameof(registries));
            if (profiles == null) throw new ArgumentNullException(nameof(profiles));
            if (tradeRoutes == null) throw new ArgumentNullException(nameof(tradeRoutes));

            List<string> errors = new List<string>();
            List<string> warnings = new List<string>();

            ValidateBuildings(packId, registries, errors, warnings);
            ValidateProfiles(profiles, errors, warnings);
            ValidateTradeRoutes(tradeRoutes, errors, warnings);
            DetectCircularTrades(tradeRoutes, errors);

            return new EconomyValidationResult(
                packId: packId,
                isValid: errors.Count == 0,
                errors: errors.AsReadOnly(),
                warnings: warnings.AsReadOnly());
        }

        /// <summary>
        /// Validates that all production buildings in the pack reference valid resource types.
        /// </summary>
        private static void ValidateBuildings(
            string packId,
            RegistryManager registries,
            List<string> errors,
            List<string> warnings)
        {
            foreach (KeyValuePair<string, RegistryEntry<BuildingDefinition>> kvp in registries.Buildings.All)
            {
                RegistryEntry<BuildingDefinition> entry = kvp.Value;
                if (!string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase))
                    continue;

                BuildingDefinition building = entry.Data;

                // Check that production keys are valid resource types
                foreach (KeyValuePair<string, int> production in building.Production)
                {
                    if (!ValidResources.Contains(production.Key))
                    {
                        errors.Add($"Building '{building.Id}' produces unknown resource type '{production.Key}'. " +
                                   $"Valid types: {string.Join(", ", ResourceRate.ValidResourceTypes)}.");
                    }

                    if (production.Value < 0)
                    {
                        warnings.Add($"Building '{building.Id}' has negative production rate ({production.Value}) " +
                                     $"for resource '{production.Key}'. This indicates consumption, not production.");
                    }
                }

                // Check economy buildings have at least one production entry
                if (string.Equals(building.BuildingType, "economy", StringComparison.OrdinalIgnoreCase)
                    && building.Production.Count == 0)
                {
                    warnings.Add($"Economy building '{building.Id}' has no production entries defined.");
                }
            }
        }

        /// <summary>
        /// Validates economy profiles for correctness.
        /// </summary>
        private static void ValidateProfiles(
            IReadOnlyList<EconomyProfile> profiles,
            List<string> errors,
            List<string> warnings)
        {
            HashSet<string> seenIds = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            foreach (EconomyProfile profile in profiles)
            {
                if (string.IsNullOrWhiteSpace(profile.Id))
                {
                    errors.Add("Economy profile has empty or missing ID.");
                    continue;
                }

                if (!seenIds.Add(profile.Id))
                {
                    errors.Add($"Duplicate economy profile ID '{profile.Id}'.");
                }

                // Validate starting resources are non-negative
                ResourceCost start = profile.StartingResources;
                if (start.Food < 0) errors.Add($"Profile '{profile.Id}' has negative starting food ({start.Food}).");
                if (start.Wood < 0) errors.Add($"Profile '{profile.Id}' has negative starting wood ({start.Wood}).");
                if (start.Stone < 0) errors.Add($"Profile '{profile.Id}' has negative starting stone ({start.Stone}).");
                if (start.Iron < 0) errors.Add($"Profile '{profile.Id}' has negative starting iron ({start.Iron}).");
                if (start.Gold < 0) errors.Add($"Profile '{profile.Id}' has negative starting gold ({start.Gold}).");

                // Validate production multipliers reference valid resources
                foreach (KeyValuePair<string, float> kvp in profile.ProductionMultipliers)
                {
                    if (!ValidResources.Contains(kvp.Key))
                    {
                        errors.Add($"Profile '{profile.Id}' has production multiplier for unknown resource '{kvp.Key}'.");
                    }
                    if (kvp.Value < 0)
                    {
                        errors.Add($"Profile '{profile.Id}' has negative production multiplier ({kvp.Value}) for '{kvp.Key}'.");
                    }
                }

                // Validate consumption multipliers reference valid resources
                foreach (KeyValuePair<string, float> kvp in profile.ConsumptionMultipliers)
                {
                    if (!ValidResources.Contains(kvp.Key))
                    {
                        errors.Add($"Profile '{profile.Id}' has consumption multiplier for unknown resource '{kvp.Key}'.");
                    }
                    if (kvp.Value < 0)
                    {
                        errors.Add($"Profile '{profile.Id}' has negative consumption multiplier ({kvp.Value}) for '{kvp.Key}'.");
                    }
                }

                // Warn about extreme modifiers
                if (profile.TradeRateModifier <= 0)
                {
                    errors.Add($"Profile '{profile.Id}' has non-positive trade rate modifier ({profile.TradeRateModifier}).");
                }
                if (profile.WorkerEfficiency <= 0)
                {
                    warnings.Add($"Profile '{profile.Id}' has non-positive worker efficiency ({profile.WorkerEfficiency}). Workers will produce nothing.");
                }
                if (profile.StorageMultiplier <= 0)
                {
                    warnings.Add($"Profile '{profile.Id}' has non-positive storage multiplier ({profile.StorageMultiplier}). No resources can be stored.");
                }
            }
        }

        /// <summary>
        /// Validates trade routes for correctness.
        /// </summary>
        private static void ValidateTradeRoutes(
            IReadOnlyList<TradeRoute> routes,
            List<string> errors,
            List<string> warnings)
        {
            HashSet<string> seenIds = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            foreach (TradeRoute route in routes)
            {
                if (string.IsNullOrWhiteSpace(route.Id))
                {
                    errors.Add("Trade route has empty or missing ID.");
                    continue;
                }

                if (!seenIds.Add(route.Id))
                {
                    errors.Add($"Duplicate trade route ID '{route.Id}'.");
                }

                // Validate source resource
                if (!ValidResources.Contains(route.SourceResource))
                {
                    errors.Add($"Trade route '{route.Id}' has invalid source resource '{route.SourceResource}'. " +
                               $"Valid types: {string.Join(", ", ResourceRate.ValidResourceTypes)}.");
                }

                // Validate target resource
                if (!ValidResources.Contains(route.TargetResource))
                {
                    errors.Add($"Trade route '{route.Id}' has invalid target resource '{route.TargetResource}'. " +
                               $"Valid types: {string.Join(", ", ResourceRate.ValidResourceTypes)}.");
                }

                // Source and target must differ
                if (string.Equals(route.SourceResource, route.TargetResource, StringComparison.OrdinalIgnoreCase))
                {
                    errors.Add($"Trade route '{route.Id}' has the same source and target resource '{route.SourceResource}'.");
                }

                // Exchange rate must be positive
                if (route.ExchangeRate <= 0)
                {
                    errors.Add($"Trade route '{route.Id}' has non-positive exchange rate ({route.ExchangeRate}).");
                }

                // Cooldown must be non-negative
                if (route.CooldownTicks < 0)
                {
                    errors.Add($"Trade route '{route.Id}' has negative cooldown ({route.CooldownTicks}).");
                }

                // Warn about extreme exchange rates
                if (route.ExchangeRate > 100)
                {
                    warnings.Add($"Trade route '{route.Id}' has very high exchange rate ({route.ExchangeRate}). " +
                                 "This trade may not be economically viable.");
                }

                if (route.ExchangeRate > 0 && route.ExchangeRate < 0.01f)
                {
                    warnings.Add($"Trade route '{route.Id}' has extremely low exchange rate ({route.ExchangeRate}). " +
                                 "This trade is almost free and may be unbalanced.");
                }
            }
        }

        /// <summary>
        /// Detects circular trade dependencies that could allow infinite resource generation.
        /// A circular dependency exists when A->B->C->A trade routes form a cycle where
        /// the combined exchange rates yield a net gain.
        /// </summary>
        private static void DetectCircularTrades(IReadOnlyList<TradeRoute> routes, List<string> errors)
        {
            // Build adjacency: resource -> list of (target_resource, exchange_rate, route_id)
            Dictionary<string, List<(string target, float rate, string routeId)>> graph =
                new Dictionary<string, List<(string, float, string)>>(StringComparer.OrdinalIgnoreCase);

            foreach (TradeRoute route in routes)
            {
                if (!route.Enabled) continue;
                if (string.IsNullOrWhiteSpace(route.SourceResource) || string.IsNullOrWhiteSpace(route.TargetResource))
                    continue;

                if (!graph.ContainsKey(route.SourceResource))
                {
                    graph[route.SourceResource] = new List<(string, float, string)>();
                }
                graph[route.SourceResource].Add((route.TargetResource, route.ExchangeRate, route.Id));
            }

            // DFS from each resource to detect profitable cycles
            foreach (string startResource in graph.Keys)
            {
                // Track visited resources in current path with cumulative exchange rate
                DetectCyclesFrom(startResource, startResource, 1.0f, new HashSet<string>(StringComparer.OrdinalIgnoreCase),
                    new List<string>(), graph, errors);
            }
        }

        private static void DetectCyclesFrom(
            string startResource,
            string currentResource,
            float cumulativeRate,
            HashSet<string> visited,
            List<string> path,
            Dictionary<string, List<(string target, float rate, string routeId)>> graph,
            List<string> errors)
        {
            if (!graph.TryGetValue(currentResource, out List<(string target, float rate, string routeId)>? edges))
                return;

            foreach ((string target, float rate, string routeId) in edges)
            {
                float newCumulativeRate = cumulativeRate * rate;

                if (string.Equals(target, startResource, StringComparison.OrdinalIgnoreCase))
                {
                    // Found a cycle. Check if it's profitable (cumulative rate < 1.0 means net gain)
                    if (newCumulativeRate < 1.0f)
                    {
                        List<string> cyclePath = new List<string>(path) { routeId };
                        errors.Add($"Profitable circular trade detected via routes [{string.Join(" -> ", cyclePath)}]. " +
                                   $"Combined exchange rate {newCumulativeRate:F3} < 1.0 allows infinite resource generation.");
                    }
                }
                else if (!visited.Contains(target) && path.Count < 10) // depth limit to prevent combinatorial explosion
                {
                    visited.Add(target);
                    path.Add(routeId);
                    DetectCyclesFrom(startResource, target, newCumulativeRate, visited, path, graph, errors);
                    path.RemoveAt(path.Count - 1);
                    visited.Remove(target);
                }
            }
        }
    }
}
