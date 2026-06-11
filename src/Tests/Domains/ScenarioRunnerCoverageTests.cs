#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Scripting;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="ScenarioRunner"/> — initialization guard, victory-condition
/// all-met evaluation (SurviveWaves), defeat-condition any-met evaluation
/// (CommandCenterDestroyed), and the empty-conditions / uninitialized edge cases.
/// </summary>
public class ScenarioRunnerCoverageTests
{
    private static ScenarioRunner InitializedWith(
        IEnumerable<VictoryCondition>? victory = null,
        IEnumerable<DefeatCondition>? defeat = null)
    {
        ScenarioDefinition scenario = new() { Id = "scn", DisplayName = "S", WaveCount = 5 };
        if (victory != null) scenario.VictoryConditions.AddRange(victory);
        if (defeat != null) scenario.DefeatConditions.AddRange(defeat);

        ScenarioRunner runner = new();
        runner.Initialize(scenario);
        return runner;
    }

    [Fact]
    public void IsInitialized_FalseBeforeInitialize_TrueAfter()
    {
        ScenarioRunner runner = new();
        runner.IsInitialized.Should().BeFalse();

        runner.Initialize(new ScenarioDefinition { Id = "x", DisplayName = "X", WaveCount = 1 });

        runner.IsInitialized.Should().BeTrue();
    }

    [Fact]
    public void Initialize_Null_Throws()
    {
        Action act = () => new ScenarioRunner().Initialize(null!);

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void CheckVictoryConditions_BeforeInitialize_Throws()
    {
        Action act = () => new ScenarioRunner().CheckVictoryConditions(new GameState());

        act.Should().Throw<InvalidOperationException>();
    }

    [Fact]
    public void CheckVictoryConditions_NoConditions_ReturnsFalse()
    {
        ScenarioRunner runner = InitializedWith();

        runner.CheckVictoryConditions(new GameState()).Should().BeFalse();
    }

    [Fact]
    public void CheckVictoryConditions_SurviveWavesMet_ReturnsTrue()
    {
        ScenarioRunner runner = InitializedWith(victory: new[]
        {
            new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 5 }
        });

        // Reached wave 5 → condition met.
        runner.CheckVictoryConditions(new GameState { CurrentWave = 5 }).Should().BeTrue();
    }

    [Fact]
    public void CheckVictoryConditions_SurviveWavesNotMet_ReturnsFalse()
    {
        ScenarioRunner runner = InitializedWith(victory: new[]
        {
            new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 5 }
        });

        runner.CheckVictoryConditions(new GameState { CurrentWave = 3 }).Should().BeFalse();
    }

    [Fact]
    public void CheckDefeatConditions_NoConditions_ReturnsFalse()
    {
        ScenarioRunner runner = InitializedWith();

        runner.CheckDefeatConditions(new GameState()).Should().BeFalse();
    }

    [Fact]
    public void CheckDefeatConditions_CommandCenterDestroyed_ReturnsTrue()
    {
        ScenarioRunner runner = InitializedWith(defeat: new[]
        {
            new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
        });

        runner.CheckDefeatConditions(new GameState { CommandCenterAlive = false }).Should().BeTrue();
    }

    [Fact]
    public void CheckDefeatConditions_CommandCenterAlive_ReturnsFalse()
    {
        ScenarioRunner runner = InitializedWith(defeat: new[]
        {
            new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
        });

        runner.CheckDefeatConditions(new GameState { CommandCenterAlive = true }).Should().BeFalse();
    }

    [Fact]
    public void CheckDefeatConditions_BeforeInitialize_Throws()
    {
        Action act = () => new ScenarioRunner().CheckDefeatConditions(new GameState());

        act.Should().Throw<InvalidOperationException>();
    }
}
