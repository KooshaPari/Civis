#nullable enable

using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Autograder;

public sealed class BridgeAcceptanceScorerTests
{
    [Fact]
    public void Tier1_requires_spec_focused_test_and_traceability_row()
    {
        BridgeAcceptanceEvidence evidence = new(
            Tier1SpecExists: true,
            Tier1FocusedTestExists: true,
            Tier1TraceabilityRowExists: true);

        BridgeTierScore score = BridgeAcceptanceScorer.ScoreTier1(evidence);

        score.Tier.Should().Be("tier-1");
        score.Pass.Should().BeTrue();
        score.Evidence.Should().BeEquivalentTo(
            "tier-1 spec",
            "tier-1 focused test",
            "tier-1 traceability row");
        score.Gaps.Should().BeEmpty();
    }

    [Fact]
    public void Tier2_requires_mutation_chaos_perf_and_load_thresholds()
    {
        BridgeAcceptanceEvidence evidence = Tier1PassEvidence() with
        {
            MutationScorePercent = BridgeAcceptanceScorer.Tier2MutationFloorPercent,
            ChaosEvidenceCount = 1,
            PerfBaselineCount = 1,
            LoadThresholdCount = 1
        };

        BridgeTierScore score = BridgeAcceptanceScorer.ScoreTier2(evidence);

        score.Tier.Should().Be("tier-2");
        score.Pass.Should().BeTrue();
        score.Gaps.Should().BeEmpty();
        score.Evidence.Should().Contain("mutation score>=85%");
        score.Evidence.Should().Contain("1+ chaos evidence");
        score.Evidence.Should().Contain("1+ perf baseline");
        score.Evidence.Should().Contain("1+ load threshold");
    }

    [Fact]
    public void Tier2_reports_missing_threshold_evidence()
    {
        BridgeTierScore score = BridgeAcceptanceScorer.ScoreTier2(Tier1PassEvidence());

        score.Pass.Should().BeFalse();
        score.Gaps.Should().Contain("mutation score below 85% (missing)");
        score.Gaps.Should().Contain("chaos evidence count below 1 (0)");
        score.Gaps.Should().Contain("perf baseline count below 1 (0)");
        score.Gaps.Should().Contain("load threshold count below 1 (0)");
    }

    [Fact]
    public void Tier3_requires_elite_bridge_build_evidence_bundle()
    {
        BridgeAcceptanceEvidence evidence = Tier1PassEvidence() with
        {
            MutationScorePercent = BridgeAcceptanceScorer.Tier2MutationFloorPercent,
            ChaosEvidenceCount = 2,
            PerfBaselineCount = 2,
            LoadThresholdCount = 2,
            BridgeCoveragePercent = BridgeAcceptanceScorer.Tier3BridgeCoverageFloorPercent,
            BddRegressionSuiteCount = 1
        };

        BridgeTierScore score = BridgeAcceptanceScorer.ScoreTier3(evidence);

        score.Tier.Should().Be("tier-3");
        score.Pass.Should().BeTrue();
        score.Gaps.Should().BeEmpty();
        score.Evidence.Should().Contain("Bridge coverage>=75%");
        score.Evidence.Should().Contain("mutation score>=70%");
        score.Evidence.Should().Contain("2+ chaos evidence");
        score.Evidence.Should().Contain("2+ perf baseline");
        score.Evidence.Should().Contain("2+ load threshold");
        score.Evidence.Should().Contain("1+ BDD regression suite");
    }

    [Fact]
    public void Tier3_reports_each_missing_elite_evidence_category()
    {
        BridgeAcceptanceEvidence evidence = Tier1PassEvidence() with
        {
            MutationScorePercent = BridgeAcceptanceScorer.Tier3MutationFloorPercent,
            ChaosEvidenceCount = 1,
            PerfBaselineCount = 1,
            LoadThresholdCount = 1,
            BridgeCoveragePercent = 74.9d
        };

        BridgeTierScore score = BridgeAcceptanceScorer.ScoreTier3(evidence);

        score.Pass.Should().BeFalse();
        score.Gaps.Should().Contain("mutation score below 85% (70%)");
        score.Gaps.Should().Contain("Bridge coverage below 75% (74.9%)");
        score.Gaps.Should().Contain("chaos evidence count below 2 (1)");
        score.Gaps.Should().Contain("perf baseline count below 2 (1)");
        score.Gaps.Should().Contain("load threshold count below 2 (1)");
        score.Gaps.Should().Contain("BDD regression suite count below 1 (0)");
    }

    private static BridgeAcceptanceEvidence Tier1PassEvidence()
    {
        return new BridgeAcceptanceEvidence(
            Tier1SpecExists: true,
            Tier1FocusedTestExists: true,
            Tier1TraceabilityRowExists: true);
    }
}
