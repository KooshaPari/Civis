using System.Collections.Generic;

namespace DINOForge.Domains.Warfare.Balance
{
    /// <summary>
    /// Report summarizing the overall power rating of a faction, including per-unit breakdowns.
    /// </summary>
    public sealed class FactionPowerReport
    {
        /// <summary>
        /// The faction identifier this report covers.
        /// </summary>
        public string FactionId { get; }

        /// <summary>
        /// The archetype identifier applied to this faction.
        /// </summary>
        public string ArchetypeId { get; }

        /// <summary>
        /// The doctrine identifier applied (if any).
        /// </summary>
        public string? DoctrineId { get; }

        /// <summary>
        /// Aggregate power rating for the entire faction (sum of unit power ratings).
        /// </summary>
        public float TotalPower { get; }

        /// <summary>
        /// Average power rating per unit.
        /// </summary>
        public float AveragePower { get; }

        /// <summary>
        /// Number of units evaluated.
        /// </summary>
        public int UnitCount { get; }

        /// <summary>
        /// Per-unit power rating breakdown (unit ID to power rating).
        /// </summary>
        public IReadOnlyDictionary<string, float> UnitPowerRatings { get; }

        /// <summary>
        /// Creates a new faction power report.
        /// </summary>
        public FactionPowerReport(
            string factionId,
            string archetypeId,
            string? doctrineId,
            float totalPower,
            float averagePower,
            int unitCount,
            IReadOnlyDictionary<string, float> unitPowerRatings)
        {
            FactionId = factionId;
            ArchetypeId = archetypeId;
            DoctrineId = doctrineId;
            TotalPower = totalPower;
            AveragePower = averagePower;
            UnitCount = unitCount;
            UnitPowerRatings = unitPowerRatings;
        }
    }
}
