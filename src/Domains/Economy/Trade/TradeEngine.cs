using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy.Models;

namespace DINOForge.Domains.Economy.Trade
{
    /// <summary>
    /// Result of evaluating a trade route's profitability and efficiency.
    /// </summary>
    public sealed class TradeEvaluation
    {
        /// <summary>
        /// The trade route that was evaluated.
        /// </summary>
        public TradeRoute Route { get; }

        /// <summary>
        /// Effective exchange rate after applying profile modifiers.
        /// Lower is better (fewer source units per target unit).
        /// </summary>
        public float EffectiveExchangeRate { get; }

        /// <summary>
        /// Efficiency score from 0.0 to 1.0. Higher means a more favorable trade.
        /// Calculated as the inverse of the effective exchange rate, normalized.
        /// </summary>
        public float Efficiency { get; }

        /// <summary>
        /// Whether this trade is profitable given current resource availability.
        /// A trade is profitable if the player has surplus of the source resource
        /// and deficit of the target resource.
        /// </summary>
        public bool IsProfitable { get; }

        /// <summary>
        /// Maximum units of target resource obtainable per execution.
        /// </summary>
        public float MaxTargetPerExecution { get; }

        /// <summary>
        /// Creates a new trade evaluation result.
        /// </summary>
        public TradeEvaluation(
            TradeRoute route,
            float effectiveExchangeRate,
            float efficiency,
            bool isProfitable,
            float maxTargetPerExecution)
        {
            Route = route;
            EffectiveExchangeRate = effectiveExchangeRate;
            Efficiency = efficiency;
            IsProfitable = isProfitable;
            MaxTargetPerExecution = maxTargetPerExecution;
        }
    }

    /// <summary>
    /// Suggestion for an optimal trade given current resource state.
    /// </summary>
    public sealed class TradeSuggestion
    {
        /// <summary>
        /// The recommended trade route.
        /// </summary>
        public TradeRoute Route { get; }

        /// <summary>
        /// Recommended amount of source resource to trade.
        /// </summary>
        public int RecommendedAmount { get; }

        /// <summary>
        /// Expected amount of target resource received.
        /// </summary>
        public float ExpectedReturn { get; }

        /// <summary>
        /// Reason this trade was recommended.
        /// </summary>
        public string Reason { get; }

        /// <summary>
        /// Creates a new trade suggestion.
        /// </summary>
        public TradeSuggestion(TradeRoute route, int recommendedAmount, float expectedReturn, string reason)
        {
            Route = route;
            RecommendedAmount = recommendedAmount;
            ExpectedReturn = expectedReturn;
            Reason = reason;
        }
    }

    /// <summary>
    /// Manages trade route evaluation and optimization. Calculates effective exchange rates,
    /// evaluates trade profitability, and suggests optimal trades based on resource state.
    /// In DINO, gold is obtained exclusively through trading, making this engine critical
    /// for late-game economy strategy.
    /// </summary>
    public sealed class TradeEngine
    {
        /// <summary>
        /// Calculates the effective exchange rate for trading between two resources,
        /// applying the economy profile's trade rate modifier.
        /// </summary>
        /// <param name="route">The trade route to calculate the rate for.</param>
        /// <param name="profile">The economy profile providing trade modifiers.</param>
        /// <returns>The effective exchange rate (units of source per unit of target). Lower is better.</returns>
        public float CalculateExchangeRate(TradeRoute route, EconomyProfile profile)
        {
            if (route == null) throw new ArgumentNullException(nameof(route));
            if (profile == null) throw new ArgumentNullException(nameof(profile));

            return route.ExchangeRate * profile.TradeRateModifier;
        }

        /// <summary>
        /// Evaluates a trade route for profitability and efficiency given current resource availability.
        /// </summary>
        /// <param name="route">The trade route to evaluate.</param>
        /// <param name="profile">The economy profile providing trade modifiers.</param>
        /// <param name="availableResources">Current resource stockpiles keyed by resource type.</param>
        /// <param name="resourceBalance">Net resource income/expense keyed by resource type. Negative means deficit.</param>
        /// <returns>A trade evaluation with efficiency and profitability metrics.</returns>
        public TradeEvaluation EvaluateTradeRoute(
            TradeRoute route,
            EconomyProfile profile,
            Dictionary<string, float> availableResources,
            Dictionary<string, float> resourceBalance)
        {
            if (route == null) throw new ArgumentNullException(nameof(route));
            if (profile == null) throw new ArgumentNullException(nameof(profile));
            if (availableResources == null) throw new ArgumentNullException(nameof(availableResources));
            if (resourceBalance == null) throw new ArgumentNullException(nameof(resourceBalance));

            float effectiveRate = CalculateExchangeRate(route, profile);

            // Efficiency: inverse of exchange rate, capped at 1.0
            // An exchange rate of 1.0 (1:1) = perfect efficiency
            float efficiency = effectiveRate > 0 ? Math.Min(1.0f / effectiveRate, 1.0f) : 0f;

            // Check profitability: source should be in surplus, target should be in deficit
            float sourceBalance = resourceBalance.TryGetValue(route.SourceResource, out float srcBal) ? srcBal : 0f;
            float targetBalance = resourceBalance.TryGetValue(route.TargetResource, out float tgtBal) ? tgtBal : 0f;
            float sourceAvailable = availableResources.TryGetValue(route.SourceResource, out float srcAvail) ? srcAvail : 0f;

            bool isProfitable = sourceBalance > 0 && targetBalance < 0 && sourceAvailable > 0 && route.Enabled;

            // Calculate max target per execution
            float maxSource = route.MaxPerTransaction > 0
                ? Math.Min(sourceAvailable, route.MaxPerTransaction)
                : sourceAvailable;
            float maxTarget = effectiveRate > 0 ? maxSource / effectiveRate : 0f;

            return new TradeEvaluation(route, effectiveRate, efficiency, isProfitable, maxTarget);
        }

        /// <summary>
        /// Analyzes all available trade routes and suggests the best trade to execute
        /// given current resource stockpiles and net balances.
        /// Prioritizes trades that address the most critical resource deficits.
        /// </summary>
        /// <param name="routes">All available trade routes.</param>
        /// <param name="profile">The economy profile providing trade modifiers.</param>
        /// <param name="availableResources">Current resource stockpiles keyed by resource type.</param>
        /// <param name="resourceBalance">Net resource income/expense keyed by resource type.</param>
        /// <returns>A list of trade suggestions ordered by priority (most urgent first). Empty if no trades are beneficial.</returns>
        public List<TradeSuggestion> GetOptimalTrades(
            IReadOnlyList<TradeRoute> routes,
            EconomyProfile profile,
            Dictionary<string, float> availableResources,
            Dictionary<string, float> resourceBalance)
        {
            if (routes == null) throw new ArgumentNullException(nameof(routes));
            if (profile == null) throw new ArgumentNullException(nameof(profile));
            if (availableResources == null) throw new ArgumentNullException(nameof(availableResources));
            if (resourceBalance == null) throw new ArgumentNullException(nameof(resourceBalance));

            List<TradeSuggestion> suggestions = new List<TradeSuggestion>();

            // Find resource deficits sorted by severity
            List<KeyValuePair<string, float>> deficits = resourceBalance
                .Where(kvp => kvp.Value < 0)
                .OrderBy(kvp => kvp.Value) // most negative first
                .ToList();

            if (deficits.Count == 0)
                return suggestions;

            foreach (KeyValuePair<string, float> deficit in deficits)
            {
                string neededResource = deficit.Key;
                float deficitAmount = Math.Abs(deficit.Value);

                // Find the best route that produces the needed resource
                TradeRoute? bestRoute = null;
                TradeEvaluation? bestEval = null;

                foreach (TradeRoute route in routes)
                {
                    if (!route.Enabled) continue;
                    if (!string.Equals(route.TargetResource, neededResource, StringComparison.OrdinalIgnoreCase))
                        continue;

                    TradeEvaluation eval = EvaluateTradeRoute(route, profile, availableResources, resourceBalance);

                    if (eval.MaxTargetPerExecution <= 0) continue;

                    if (bestEval == null || eval.Efficiency > bestEval.Efficiency)
                    {
                        bestRoute = route;
                        bestEval = eval;
                    }
                }

                if (bestRoute != null && bestEval != null)
                {
                    float effectiveRate = bestEval.EffectiveExchangeRate;
                    // Recommend trading enough to cover the deficit, capped by what's available
                    int recommendedSource = (int)Math.Ceiling(deficitAmount * effectiveRate);
                    float sourceAvailable = availableResources.TryGetValue(bestRoute.SourceResource, out float avail) ? avail : 0f;
                    recommendedSource = Math.Min(recommendedSource, (int)sourceAvailable);

                    if (bestRoute.MaxPerTransaction > 0)
                    {
                        recommendedSource = Math.Min(recommendedSource, bestRoute.MaxPerTransaction);
                    }

                    float expectedReturn = effectiveRate > 0 ? recommendedSource / effectiveRate : 0f;

                    string reason = $"Deficit of {deficitAmount:F1} {neededResource}/tick. " +
                                    $"Trade {recommendedSource} {bestRoute.SourceResource} at {effectiveRate:F1}:1 rate.";

                    suggestions.Add(new TradeSuggestion(bestRoute, recommendedSource, expectedReturn, reason));
                }
            }

            return suggestions;
        }
    }
}
