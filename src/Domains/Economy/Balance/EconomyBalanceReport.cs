using System.Collections.Generic;

namespace DINOForge.Domains.Economy.Balance
{
    /// <summary>
    /// Per-faction economy summary within a balance report.
    /// </summary>
    public sealed class FactionEconomySummary
    {
        /// <summary>
        /// The faction identifier.
        /// </summary>
        public string FactionId { get; }

        /// <summary>
        /// Per-resource production rates (units per tick).
        /// </summary>
        public IReadOnlyDictionary<string, float> Production { get; }

        /// <summary>
        /// Per-resource consumption rates (units per tick).
        /// </summary>
        public IReadOnlyDictionary<string, float> Consumption { get; }

        /// <summary>
        /// Net resource balance (production minus consumption) per resource type.
        /// Positive means surplus; negative means deficit.
        /// </summary>
        public IReadOnlyDictionary<string, float> NetBalance { get; }

        /// <summary>
        /// Trade efficiency score from 0.0 to 1.0. Higher means the faction
        /// has access to favorable trade routes for covering its deficits.
        /// </summary>
        public float TradeEfficiency { get; }

        /// <summary>
        /// Sustainability score from 0.0 to 1.0. A score of 1.0 means the faction
        /// can fully sustain itself without any external trade. Scores below 1.0
        /// indicate reliance on trade to cover resource deficits.
        /// </summary>
        public float SustainabilityScore { get; }

        /// <summary>
        /// Number of resources in deficit (negative net balance).
        /// </summary>
        public int DeficitCount { get; }

        /// <summary>
        /// Number of resources in surplus (positive net balance).
        /// </summary>
        public int SurplusCount { get; }

        /// <summary>
        /// Creates a new faction economy summary.
        /// </summary>
        public FactionEconomySummary(
            string factionId,
            IReadOnlyDictionary<string, float> production,
            IReadOnlyDictionary<string, float> consumption,
            IReadOnlyDictionary<string, float> netBalance,
            float tradeEfficiency,
            float sustainabilityScore,
            int deficitCount,
            int surplusCount)
        {
            FactionId = factionId;
            Production = production;
            Consumption = consumption;
            NetBalance = netBalance;
            TradeEfficiency = tradeEfficiency;
            SustainabilityScore = sustainabilityScore;
            DeficitCount = deficitCount;
            SurplusCount = surplusCount;
        }
    }

    /// <summary>
    /// Comprehensive economy balance analysis report across all factions in a pack.
    /// Includes per-faction summaries, trade metrics, and sustainability assessments.
    /// </summary>
    public sealed class EconomyBalanceReport
    {
        /// <summary>
        /// The pack identifier this report was generated for.
        /// </summary>
        public string PackId { get; }

        /// <summary>
        /// Per-faction economy summaries.
        /// </summary>
        public IReadOnlyDictionary<string, FactionEconomySummary> FactionSummaries { get; }

        /// <summary>
        /// Overall balance score from 0.0 to 1.0. Measures how evenly matched
        /// faction economies are. 1.0 means perfectly balanced.
        /// </summary>
        public float OverallBalanceScore { get; }

        /// <summary>
        /// Warnings about economy balance issues (e.g. "Faction X has no food production").
        /// </summary>
        public IReadOnlyList<string> Warnings { get; }

        /// <summary>
        /// Creates a new economy balance report.
        /// </summary>
        public EconomyBalanceReport(
            string packId,
            IReadOnlyDictionary<string, FactionEconomySummary> factionSummaries,
            float overallBalanceScore,
            IReadOnlyList<string> warnings)
        {
            PackId = packId;
            FactionSummaries = factionSummaries;
            OverallBalanceScore = overallBalanceScore;
            Warnings = warnings;
        }
    }
}
