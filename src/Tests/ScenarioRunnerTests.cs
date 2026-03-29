#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Scripting;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for <see cref="ScenarioRunner"/> — victory/defeat condition evaluation,
/// event triggering, and initialization lifecycle.
/// </summary>
public class ScenarioRunnerTests
{
    // ─── Initialization ───────────────────────────────────────────────────────

    [Fact]
    public void Initialize_SetsIsInitializedAndCurrentScenario()
    {
        ScenarioRunner runner = new ScenarioRunner();
        ScenarioDefinition def = MakeScenario("s1");

        runner.IsInitialized.Should().BeFalse();
        runner.Initialize(def);
        runner.IsInitialized.Should().BeTrue();
        runner.CurrentScenario.Should().BeSameAs(def);
    }

    [Fact]
    public void Initialize_Null_ThrowsArgumentNullException()
    {
        ScenarioRunner runner = new ScenarioRunner();
        Action act = () => runner.Initialize(null!);
        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void Initialize_ResetsAlreadyFiredEvents()
    {
        ScenarioRunner runner = new ScenarioRunner();
        ScenarioDefinition def = MakeScenario("s1", events: new List<ScriptedEvent>
        {
            new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnWave, TriggerValue = 1 }
        });
        runner.Initialize(def);

        // Fire the event by evaluating GetPendingEvents at wave 1
        GameState gs = new GameState { CurrentWave = 1 };
        runner.GetPendingEvents(gs); // this fires evt1

        // Re-initialize should reset fired event tracking
        runner.Initialize(def);
        runner.GetPendingEvents(gs).Should().HaveCount(1, "re-init resets fired event set");
    }

    // ─── Victory Conditions ───────────────────────────────────────────────────

    [Fact]
    public void CheckVictoryConditions_NotInitialized_ThrowsInvalidOperationException()
    {
        ScenarioRunner runner = new ScenarioRunner();
        Action act = () => runner.CheckVictoryConditions(new GameState());
        act.Should().Throw<InvalidOperationException>();
    }

    [Fact]
    public void CheckVictoryConditions_NoConditions_ReturnsFalse()
    {
        ScenarioRunner runner = InitWith(MakeScenario("s1"));
        runner.CheckVictoryConditions(new GameState()).Should().BeFalse();
    }

    [Fact]
    public void CheckVictoryConditions_SurviveWaves_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 5 }
        });
        ScenarioRunner runner = InitWith(def);
        runner.CheckVictoryConditions(new GameState { CurrentWave = 5 }).Should().BeTrue();
    }

    [Fact]
    public void CheckVictoryConditions_SurviveWaves_NotYetMet_ReturnsFalse()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 5 }
        });
        ScenarioRunner runner = InitWith(def);
        runner.CheckVictoryConditions(new GameState { CurrentWave = 3 }).Should().BeFalse();
    }

    [Fact]
    public void CheckVictoryConditions_ReachPopulation_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.ReachPopulation, TargetValue = 100 }
        });
        InitWith(def).CheckVictoryConditions(new GameState { Population = 100 }).Should().BeTrue();
    }

    [Fact]
    public void CheckVictoryConditions_AccumulateResource_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.AccumulateResource, TargetValue = 500, TargetId = "gold" }
        });
        GameState gs = new GameState();
        gs.Resources["gold"] = 600;
        InitWith(def).CheckVictoryConditions(gs).Should().BeTrue();
    }

    [Fact]
    public void CheckVictoryConditions_AccumulateResource_NotMet_ReturnsFalse()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.AccumulateResource, TargetValue = 500, TargetId = "gold" }
        });
        GameState gs = new GameState();
        gs.Resources["gold"] = 100;
        InitWith(def).CheckVictoryConditions(gs).Should().BeFalse();
    }

    [Fact]
    public void CheckVictoryConditions_TimeSurvival_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.TimeSurvival, TargetValue = 300 }
        });
        InitWith(def).CheckVictoryConditions(new GameState { ElapsedSeconds = 300 }).Should().BeTrue();
    }

    [Fact]
    public void CheckVictoryConditions_AllConditionsMustBeMetForTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", victory: new List<VictoryCondition>
        {
            new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 5 },
            new VictoryCondition { ConditionType = VictoryConditionType.ReachPopulation, TargetValue = 50 }
        });
        ScenarioRunner runner = InitWith(def);

        // Waves met but not population
        runner.CheckVictoryConditions(new GameState { CurrentWave = 5, Population = 10 }).Should().BeFalse();

        // Both met
        runner.CheckVictoryConditions(new GameState { CurrentWave = 5, Population = 50 }).Should().BeTrue();
    }

    // ─── Defeat Conditions ────────────────────────────────────────────────────

    [Fact]
    public void CheckDefeatConditions_NotInitialized_ThrowsInvalidOperationException()
    {
        ScenarioRunner runner = new ScenarioRunner();
        Action act = () => runner.CheckDefeatConditions(new GameState());
        act.Should().Throw<InvalidOperationException>();
    }

    [Fact]
    public void CheckDefeatConditions_NoConditions_ReturnsFalse()
    {
        InitWith(MakeScenario("s1")).CheckDefeatConditions(new GameState()).Should().BeFalse();
    }

    [Fact]
    public void CheckDefeatConditions_CommandCenterDestroyed_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", defeat: new List<DefeatCondition>
        {
            new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
        });
        InitWith(def).CheckDefeatConditions(new GameState { CommandCenterAlive = false }).Should().BeTrue();
    }

    [Fact]
    public void CheckDefeatConditions_CommandCenterAlive_ReturnsFalse()
    {
        ScenarioDefinition def = MakeScenario("s1", defeat: new List<DefeatCondition>
        {
            new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
        });
        InitWith(def).CheckDefeatConditions(new GameState { CommandCenterAlive = true }).Should().BeFalse();
    }

    [Fact]
    public void CheckDefeatConditions_PopulationZero_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", defeat: new List<DefeatCondition>
        {
            new DefeatCondition { ConditionType = DefeatConditionType.PopulationZero }
        });
        InitWith(def).CheckDefeatConditions(new GameState { Population = 0 }).Should().BeTrue();
    }

    [Fact]
    public void CheckDefeatConditions_TimeExpired_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", defeat: new List<DefeatCondition>
        {
            new DefeatCondition { ConditionType = DefeatConditionType.TimeExpired, TargetValue = 600 }
        });
        InitWith(def).CheckDefeatConditions(new GameState { ElapsedSeconds = 601 }).Should().BeTrue();
    }

    [Fact]
    public void CheckDefeatConditions_OneOfMultiple_Met_ReturnsTrue()
    {
        ScenarioDefinition def = MakeScenario("s1", defeat: new List<DefeatCondition>
        {
            new DefeatCondition { ConditionType = DefeatConditionType.PopulationZero },
            new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
        });
        // Only command center dead — but that's one of the conditions, so defeat
        InitWith(def).CheckDefeatConditions(new GameState { Population = 10, CommandCenterAlive = false }).Should().BeTrue();
    }

    // ─── Scripted Events ──────────────────────────────────────────────────────

    [Fact]
    public void GetPendingEvents_OnWave_TriggersAtCorrectWave()
    {
        ScenarioDefinition def = MakeScenario("s1", events: new List<ScriptedEvent>
        {
            new ScriptedEvent { Id = "wave3-event", TriggerType = TriggerType.OnWave, TriggerValue = 3 }
        });
        ScenarioRunner runner = InitWith(def);

        runner.GetPendingEvents(new GameState { CurrentWave = 2 }).Should().BeEmpty();
        runner.GetPendingEvents(new GameState { CurrentWave = 3 }).Should().ContainSingle(e => e.Id == "wave3-event");
    }

    [Fact]
    public void GetPendingEvents_FiredEventNotReturnedTwice()
    {
        ScenarioDefinition def = MakeScenario("s1", events: new List<ScriptedEvent>
        {
            new ScriptedEvent { Id = "once", TriggerType = TriggerType.OnWave, TriggerValue = 1 }
        });
        ScenarioRunner runner = InitWith(def);

        runner.GetPendingEvents(new GameState { CurrentWave = 1 }).Should().HaveCount(1);
        runner.GetPendingEvents(new GameState { CurrentWave = 1 }).Should().BeEmpty("already fired");
    }

    [Fact]
    public void GetPendingEvents_OnTime_TriggersAtThreshold()
    {
        ScenarioDefinition def = MakeScenario("s1", events: new List<ScriptedEvent>
        {
            new ScriptedEvent { Id = "t60", TriggerType = TriggerType.OnTime, TriggerValue = 60 }
        });
        ScenarioRunner runner = InitWith(def);

        runner.GetPendingEvents(new GameState { ElapsedSeconds = 59 }).Should().BeEmpty();
        runner.GetPendingEvents(new GameState { ElapsedSeconds = 60 }).Should().ContainSingle();
    }

    [Fact]
    public void ResetEvents_AllowsFiringAgain()
    {
        ScenarioDefinition def = MakeScenario("s1", events: new List<ScriptedEvent>
        {
            new ScriptedEvent { Id = "ev", TriggerType = TriggerType.OnWave, TriggerValue = 1 }
        });
        ScenarioRunner runner = InitWith(def);
        runner.GetPendingEvents(new GameState { CurrentWave = 1 });

        runner.ResetEvents();
        runner.GetPendingEvents(new GameState { CurrentWave = 1 }).Should().HaveCount(1);
    }

    // ─── Helpers ──────────────────────────────────────────────────────────────

    private static ScenarioRunner InitWith(ScenarioDefinition def)
    {
        ScenarioRunner r = new ScenarioRunner();
        r.Initialize(def);
        return r;
    }

    private static ScenarioDefinition MakeScenario(
        string id,
        List<VictoryCondition>? victory = null,
        List<DefeatCondition>? defeat = null,
        List<ScriptedEvent>? events = null)
    {
        return new ScenarioDefinition
        {
            Id = id,
            DisplayName = id,
            WaveCount = 10,
            VictoryConditions = victory ?? new List<VictoryCondition>(),
            DefeatConditions = defeat ?? new List<DefeatCondition>(),
            ScriptedEvents = events ?? new List<ScriptedEvent>()
        };
    }
}
