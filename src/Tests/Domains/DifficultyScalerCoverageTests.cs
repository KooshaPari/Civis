#nullable enable
using System;
using DINOForge.Domains.Scenario.Balance;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="DifficultyScaler"/> — the per-difficulty multiplier table,
/// resource scaling (with zero-clamp + null guard), and wave-intensity ramp (with the
/// positive-wave-number guard). Pure Scenario-domain math.
/// </summary>
public class DifficultyScalerCoverageTests
{
    [Theory]
    [InlineData(Difficulty.Easy, 1.5f)]
    [InlineData(Difficulty.Normal, 1.0f)]
    [InlineData(Difficulty.Hard, 0.7f)]
    [InlineData(Difficulty.Nightmare, 0.5f)]
    public void GetDifficultyMultiplier_ReturnsExpectedPerDifficulty(Difficulty difficulty, float expected)
    {
        new DifficultyScaler().GetDifficultyMultiplier(difficulty).Should().Be(expected);
    }

    [Fact]
    public void ScaleResources_Easy_GivesBonus()
    {
        ResourceCost baseCost = new() { Food = 100, Gold = 50 };

        ResourceCost scaled = new DifficultyScaler().ScaleResources(baseCost, Difficulty.Easy);

        scaled.Food.Should().Be(150); // 100 * 1.5
        scaled.Gold.Should().Be(75);  // 50 * 1.5
    }

    [Fact]
    public void ScaleResources_Nightmare_HalvesResources()
    {
        ResourceCost baseCost = new() { Wood = 200, Stone = 80 };

        ResourceCost scaled = new DifficultyScaler().ScaleResources(baseCost, Difficulty.Nightmare);

        scaled.Wood.Should().Be(100); // 200 * 0.5
        scaled.Stone.Should().Be(40); // 80 * 0.5
    }

    [Fact]
    public void ScaleResources_ProducesNewInstance_NotMutatingInput()
    {
        ResourceCost baseCost = new() { Food = 100 };

        ResourceCost scaled = new DifficultyScaler().ScaleResources(baseCost, Difficulty.Hard);

        baseCost.Food.Should().Be(100);     // original untouched
        scaled.Should().NotBeSameAs(baseCost);
        scaled.Food.Should().Be(70);        // 100 * 0.7 truncated
    }

    [Fact]
    public void ScaleResources_NullBase_Throws()
    {
        Action act = () => new DifficultyScaler().ScaleResources(null!, Difficulty.Normal);

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ScaleWaveIntensity_FirstWave_HasNoWaveRamp()
    {
        // Wave 1 → waveScaling 1.0 → adjusted 1.0; result == baseIntensity * difficultyFactor.
        float normalWave1 = new DifficultyScaler().ScaleWaveIntensity(1.0f, Difficulty.Normal, 1);
        float normalWave3 = new DifficultyScaler().ScaleWaveIntensity(1.0f, Difficulty.Normal, 3);

        normalWave3.Should().BeGreaterThan(normalWave1); // later waves ramp up
    }

    [Fact]
    public void ScaleWaveIntensity_HarderDifficulty_IsMoreIntense()
    {
        DifficultyScaler scaler = new();

        float normal = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 2);
        float nightmare = scaler.ScaleWaveIntensity(1.0f, Difficulty.Nightmare, 2);

        nightmare.Should().BeGreaterThan(normal); // harder → higher enemy intensity
    }

    [Theory]
    [InlineData(0)]
    [InlineData(-1)]
    public void ScaleWaveIntensity_NonPositiveWaveNumber_Throws(int waveNumber)
    {
        Action act = () => new DifficultyScaler().ScaleWaveIntensity(1.0f, Difficulty.Normal, waveNumber);

        act.Should().Throw<ArgumentOutOfRangeException>();
    }
}
