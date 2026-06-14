#nullable enable

namespace DINOForge.Tests.Autograder;

internal sealed record BridgeAcceptanceEvidence(
    bool Tier1SpecExists,
    bool Tier1FocusedTestExists,
    bool Tier1TraceabilityRowExists,
    double? MutationScorePercent = null,
    int ChaosEvidenceCount = 0,
    int PerfBaselineCount = 0,
    int LoadThresholdCount = 0,
    double? BridgeCoveragePercent = null,
    int BddRegressionSuiteCount = 0);

internal sealed record BridgeTierScore(
    string Tier,
    bool Pass,
    IReadOnlyList<string> Evidence,
    IReadOnlyList<string> Gaps);

internal static class BridgeAcceptanceScorer
{
    public const double Tier2MutationFloorPercent = 85d;
    public const double Tier3BridgeCoverageFloorPercent = 75d;
    public const double Tier3MutationFloorPercent = 70d;

    public static BridgeTierScore ScoreTier1(BridgeAcceptanceEvidence evidence)
    {
        List<string> present = [];
        List<string> gaps = [];

        AddRequired(evidence.Tier1SpecExists, "tier-1 spec", present, gaps);
        AddRequired(evidence.Tier1FocusedTestExists, "tier-1 focused test", present, gaps);
        AddRequired(evidence.Tier1TraceabilityRowExists, "tier-1 traceability row", present, gaps);

        return new BridgeTierScore("tier-1", gaps.Count == 0, present, gaps);
    }

    public static BridgeTierScore ScoreTier2(BridgeAcceptanceEvidence evidence)
    {
        BridgeTierScore tier1 = ScoreTier1(evidence);
        List<string> present = [.. tier1.Evidence];
        List<string> gaps = [.. tier1.Gaps];

        AddThreshold(
            evidence.MutationScorePercent,
            Tier2MutationFloorPercent,
            "mutation score",
            present,
            gaps);
        AddCount(evidence.ChaosEvidenceCount, 1, "chaos evidence", present, gaps);
        AddCount(evidence.PerfBaselineCount, 1, "perf baseline", present, gaps);
        AddCount(evidence.LoadThresholdCount, 1, "load threshold", present, gaps);

        return new BridgeTierScore("tier-2", gaps.Count == 0, present, gaps);
    }

    public static BridgeTierScore ScoreTier3(BridgeAcceptanceEvidence evidence)
    {
        BridgeTierScore tier2 = ScoreTier2(evidence);
        List<string> present = [.. tier2.Evidence];
        List<string> gaps = [.. tier2.Gaps];

        AddThreshold(
            evidence.BridgeCoveragePercent,
            Tier3BridgeCoverageFloorPercent,
            "Bridge coverage",
            present,
            gaps);
        AddThreshold(
            evidence.MutationScorePercent,
            Tier3MutationFloorPercent,
            "mutation score",
            present,
            gaps);
        AddCount(evidence.ChaosEvidenceCount, 2, "chaos evidence", present, gaps);
        AddCount(evidence.PerfBaselineCount, 2, "perf baseline", present, gaps);
        AddCount(evidence.LoadThresholdCount, 2, "load threshold", present, gaps);
        AddCount(evidence.BddRegressionSuiteCount, 1, "BDD regression suite", present, gaps);

        return new BridgeTierScore("tier-3", gaps.Count == 0, present, gaps);
    }

    private static void AddRequired(
        bool exists,
        string label,
        ICollection<string> present,
        ICollection<string> gaps)
    {
        if (exists)
        {
            present.Add(label);
            return;
        }

        gaps.Add($"missing {label}");
    }

    private static void AddThreshold(
        double? actual,
        double required,
        string label,
        ICollection<string> present,
        ICollection<string> gaps)
    {
        if (actual >= required)
        {
            present.Add($"{label}>={required:0.#}%");
            return;
        }

        string actualText = actual.HasValue ? $"{actual.Value:0.#}%" : "missing";
        gaps.Add($"{label} below {required:0.#}% ({actualText})");
    }

    private static void AddCount(
        int actual,
        int required,
        string label,
        ICollection<string> present,
        ICollection<string> gaps)
    {
        if (actual >= required)
        {
            present.Add($"{required}+ {label}");
            return;
        }

        gaps.Add($"{label} count below {required} ({actual})");
    }
}
