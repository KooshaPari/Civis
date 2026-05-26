namespace DINOForge.Domains.Warfare.Balance
{
    /// <summary>
    /// Report comparing the power ratings of two factions.
    /// </summary>
    public sealed class BalanceComparisonReport
    {
        /// <summary>
        /// Power report for the first faction.
        /// </summary>
        public FactionPowerReport Faction1 { get; }

        /// <summary>
        /// Power report for the second faction.
        /// </summary>
        public FactionPowerReport Faction2 { get; }

        /// <summary>
        /// Power delta (Faction1.TotalPower - Faction2.TotalPower).
        /// Positive means faction 1 is stronger.
        /// </summary>
        public float PowerDelta { get; }

        /// <summary>
        /// Power ratio (Faction1.TotalPower / Faction2.TotalPower).
        /// 1.0 means perfectly balanced. Greater than 1.0 means faction 1 is stronger.
        /// </summary>
        public float PowerRatio { get; }

        /// <summary>
        /// Assessment label: "balanced", "slight_advantage", or "significant_advantage".
        /// </summary>
        public string Assessment { get; }

        /// <summary>
        /// Which faction has the advantage (faction ID), or null if balanced.
        /// </summary>
        public string? StrongerFaction { get; }

        /// <summary>
        /// Creates a new balance comparison report.
        /// </summary>
        public BalanceComparisonReport(
            FactionPowerReport faction1,
            FactionPowerReport faction2,
            float powerDelta,
            float powerRatio,
            string assessment,
            string? strongerFaction)
        {
            Faction1 = faction1;
            Faction2 = faction2;
            PowerDelta = powerDelta;
            PowerRatio = powerRatio;
            Assessment = assessment;
            StrongerFaction = strongerFaction;
        }
    }
}
