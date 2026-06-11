#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Waves;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="WaveComposer"/> — <c>ScaleWave</c> difficulty math and
/// <c>ComposeWaves</c> guard/empty-roster branches. Pure Warfare-domain logic, no ECS.
/// </summary>
public class WaveComposerCoverageTests
{
    [Fact]
    public void ScaleWave_DoublesSpawnCounts_AndScalesDifficulty()
    {
        WaveDefinition baseWave = new()
        {
            Id = "w1",
            WaveNumber = 1,
            SpawnGroups = new List<SpawnGroup>
            {
                new() { UnitId = "u_a", Count = 4, SpawnDelay = 2f, SpawnPoint = "north" }
            },
            DifficultyScaling = new DifficultyScaling
            {
                CountMultiplier = 1f, HealthMultiplier = 1f, DamageMultiplier = 1f
            }
        };

        WaveDefinition scaled = new WaveComposer().ScaleWave(baseWave, 2.0f);

        scaled.Id.Should().Be("w1");
        scaled.SpawnGroups.Should().ContainSingle();
        scaled.SpawnGroups[0].Count.Should().Be(8); // 4 * 2.0
        scaled.SpawnGroups[0].UnitId.Should().Be("u_a");
        scaled.DifficultyScaling!.CountMultiplier.Should().Be(2f);
        scaled.DifficultyScaling.HealthMultiplier.Should().Be(2f);
        scaled.DifficultyScaling.DamageMultiplier.Should().Be(2f);
    }

    [Fact]
    public void ScaleWave_NeverScalesCountBelowOne()
    {
        WaveDefinition baseWave = new()
        {
            Id = "w1",
            SpawnGroups = new List<SpawnGroup> { new() { UnitId = "u", Count = 1 } }
        };

        WaveDefinition scaled = new WaveComposer().ScaleWave(baseWave, 0.1f); // 1 * 0.1 = 0 → clamped to 1

        scaled.SpawnGroups[0].Count.Should().Be(1);
    }

    [Fact]
    public void ScaleWave_NullDifficultyScaling_TreatsBaseAsOne()
    {
        WaveDefinition baseWave = new()
        {
            Id = "w1",
            SpawnGroups = new List<SpawnGroup>(),
            DifficultyScaling = null
        };

        WaveDefinition scaled = new WaveComposer().ScaleWave(baseWave, 3.0f);

        scaled.DifficultyScaling!.CountMultiplier.Should().Be(3f); // (null→1) * 3
    }

    [Fact]
    public void ScaleWave_NullBaseWave_Throws()
    {
        Action act = () => new WaveComposer().ScaleWave(null!, 2f);

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ComposeWaves_NoUnits_ReturnsPlaceholderWavesWithFinalFlagged()
    {
        FactionDefinition faction = new() { Faction = new FactionInfo { Id = "fac" } };
        Registry<UnitDefinition> emptyUnits = new();

        IReadOnlyList<WaveDefinition> waves = new WaveComposer().ComposeWaves(faction, emptyUnits, waveCount: 3);

        waves.Should().HaveCount(3);
        waves[0].WaveNumber.Should().Be(1);
        waves[0].IsFinalWave.Should().BeFalse();
        waves[2].IsFinalWave.Should().BeTrue();
        waves[2].Id.Should().Be("fac_wave_3");
    }

    [Fact]
    public void ComposeWaves_NonPositiveWaveCount_Throws()
    {
        FactionDefinition faction = new() { Faction = new FactionInfo { Id = "fac" } };

        Action act = () => new WaveComposer().ComposeWaves(faction, new Registry<UnitDefinition>(), waveCount: 0);

        act.Should().Throw<ArgumentOutOfRangeException>();
    }

    [Fact]
    public void ComposeWaves_NullArgs_Throw()
    {
        FactionDefinition faction = new() { Faction = new FactionInfo { Id = "fac" } };

        Action nullFaction = () => new WaveComposer().ComposeWaves(null!, new Registry<UnitDefinition>(), 1);
        Action nullUnits = () => new WaveComposer().ComposeWaves(faction, null!, 1);

        nullFaction.Should().Throw<ArgumentNullException>();
        nullUnits.Should().Throw<ArgumentNullException>();
    }
}
