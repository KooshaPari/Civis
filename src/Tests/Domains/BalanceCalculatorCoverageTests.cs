#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Balance;
using DINOForge.Domains.Warfare.Doctrines;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="BalanceCalculator.CompareFactions"/> — the ratio-based
/// balance assessment branches (balanced / slight_advantage / significant_advantage)
/// and the argument-null guards. Exercises pure Warfare-domain balance logic with no ECS.
/// </summary>
public class BalanceCalculatorCoverageTests
{
    private static BalanceCalculator NewCalculator() => new(new DoctrineEngine());

    private static FactionPowerReport Report(string id, float totalPower) => new(
        factionId: id,
        archetypeId: "arch",
        doctrineId: null,
        totalPower: totalPower,
        averagePower: totalPower,
        unitCount: 1,
        unitPowerRatings: new Dictionary<string, float> { [id] = totalPower });

    [Fact]
    public void CompareFactions_NearEqualPower_IsBalancedWithNoStronger()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 100f), Report("b", 105f)); // ratio ~0.952 → <0.10 from 1.0

        result.Assessment.Should().Be("balanced");
        result.StrongerFaction.Should().BeNull();
    }

    [Fact]
    public void CompareFactions_ModerateGap_IsSlightAdvantageToStronger()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 120f), Report("b", 100f)); // ratio 1.2 → absDelta 0.20 → slight

        result.Assessment.Should().Be("slight_advantage");
        result.StrongerFaction.Should().Be("a");
    }

    [Fact]
    public void CompareFactions_LargeGap_IsSignificantAdvantage()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 100f), Report("b", 200f)); // ratio 0.5 → absDelta 0.5 → significant

        result.Assessment.Should().Be("significant_advantage");
        result.StrongerFaction.Should().Be("b"); // delta negative → faction2 stronger
    }

    [Fact]
    public void CompareFactions_PopulatesDeltaAndRatio()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 150f), Report("b", 100f));

        result.PowerDelta.Should().Be(50f);
        result.PowerRatio.Should().BeApproximately(1.5f, 0.001f);
        result.Faction1.FactionId.Should().Be("a");
        result.Faction2.FactionId.Should().Be("b");
    }

    [Fact]
    public void CompareFactions_ZeroSecondPower_UsesMaxRatioFallback()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 100f), Report("b", 0f));

        result.PowerRatio.Should().Be(float.MaxValue);
        result.Assessment.Should().Be("significant_advantage");
        result.StrongerFaction.Should().Be("a");
    }

    [Fact]
    public void CompareFactions_BothZero_IsBalanced()
    {
        BalanceComparisonReport result = NewCalculator()
            .CompareFactions(Report("a", 0f), Report("b", 0f)); // ratio fallback 1.0 → balanced

        result.PowerRatio.Should().Be(1.0f);
        result.Assessment.Should().Be("balanced");
    }

    [Fact]
    public void CompareFactions_NullFaction1_Throws()
    {
        Action act = () => NewCalculator().CompareFactions(null!, Report("b", 100f));

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void CompareFactions_NullFaction2_Throws()
    {
        Action act = () => NewCalculator().CompareFactions(Report("a", 100f), null!);

        act.Should().Throw<ArgumentNullException>();
    }
}
