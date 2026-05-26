using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario;
using DINOForge.Domains.Scenario.Balance;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Scripting;
using DINOForge.Domains.Scenario.Validation;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

// EventAction is in DINOForge.Domains.Scenario.Models

namespace DINOForge.Tests
{
    public class ScenarioTests
    {
        // ── DifficultyScaler ────────────────────────────────

        [Theory]
        [InlineData(Difficulty.Easy, 1.5f)]
        [InlineData(Difficulty.Normal, 1.0f)]
        [InlineData(Difficulty.Hard, 0.7f)]
        [InlineData(Difficulty.Nightmare, 0.5f)]
        public void DifficultyScaler_GetMultiplier_ReturnsCorrectValues(Difficulty difficulty, float expected)
        {
            DifficultyScaler scaler = new DifficultyScaler();
            scaler.GetDifficultyMultiplier(difficulty).Should().Be(expected);
        }

        [Fact]
        public void DifficultyScaler_ScaleResources_ScalesByDifficulty()
        {
            DifficultyScaler scaler = new DifficultyScaler();
            ResourceCost baseResources = new ResourceCost
            {
                Food = 100,
                Wood = 200,
                Stone = 50,
                Iron = 30,
                Gold = 10
            };

            ResourceCost easy = scaler.ScaleResources(baseResources, Difficulty.Easy);
            easy.Food.Should().Be(150); // 100 * 1.5
            easy.Wood.Should().Be(300); // 200 * 1.5

            ResourceCost hard = scaler.ScaleResources(baseResources, Difficulty.Hard);
            hard.Food.Should().Be(70); // 100 * 0.7
        }

        [Fact]
        public void DifficultyScaler_ScaleResources_ClampsToZero()
        {
            DifficultyScaler scaler = new DifficultyScaler();
            ResourceCost baseResources = new ResourceCost { Food = 0, Wood = 0 };

            ResourceCost result = scaler.ScaleResources(baseResources, Difficulty.Nightmare);
            result.Food.Should().BeGreaterOrEqualTo(0);
            result.Wood.Should().BeGreaterOrEqualTo(0);
        }

        [Fact]
        public void DifficultyScaler_ScaleWaveIntensity_IncreasesWithWaveNumber()
        {
            DifficultyScaler scaler = new DifficultyScaler();
            float wave1 = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 1);
            float wave5 = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 5);
            float wave10 = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 10);

            wave5.Should().BeGreaterThan(wave1);
            wave10.Should().BeGreaterThan(wave5);
        }

        [Fact]
        public void DifficultyScaler_ScaleWaveIntensity_HarderDifficultyIsMoreIntense()
        {
            DifficultyScaler scaler = new DifficultyScaler();
            float easy = scaler.ScaleWaveIntensity(1.0f, Difficulty.Easy, 5);
            float normal = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 5);
            float hard = scaler.ScaleWaveIntensity(1.0f, Difficulty.Hard, 5);

            hard.Should().BeGreaterThan(normal);
            normal.Should().BeGreaterThan(easy);
        }

        [Fact]
        public void DifficultyScaler_ScaleWaveIntensity_RejectsZeroWave()
        {
            DifficultyScaler scaler = new DifficultyScaler();
            Action act = () => scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, 0);
            act.Should().Throw<ArgumentOutOfRangeException>();
        }

        // ── ScenarioRunner ──────────────────────────────────

        [Fact]
        public void ScenarioRunner_Initialize_SetsCurrentScenario()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "test" };

            runner.Initialize(scenario);

            runner.IsInitialized.Should().BeTrue();
            runner.CurrentScenario!.Id.Should().Be("test");
        }

        [Fact]
        public void ScenarioRunner_CheckVictory_ThrowsIfNotInitialized()
        {
            ScenarioRunner runner = new ScenarioRunner();
            GameState state = new GameState();

            Action act = () => runner.CheckVictoryConditions(state);
            act.Should().Throw<InvalidOperationException>();
        }

        [Fact]
        public void ScenarioRunner_CheckVictory_SurviveWaves_TriggersAtTargetWave()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "survive-test",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 10 }
                }
            };
            runner.Initialize(scenario);

            GameState earlyState = new GameState { CurrentWave = 5 };
            runner.CheckVictoryConditions(earlyState).Should().BeFalse();

            GameState lateState = new GameState { CurrentWave = 10 };
            runner.CheckVictoryConditions(lateState).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_CheckVictory_ReachPopulation()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "pop-test",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.ReachPopulation, TargetValue = 100 }
                }
            };
            runner.Initialize(scenario);

            GameState state = new GameState { Population = 100 };
            runner.CheckVictoryConditions(state).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_CheckVictory_AccumulateResource()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "resource-test",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition
                    {
                        ConditionType = VictoryConditionType.AccumulateResource,
                        TargetValue = 5000,
                        TargetId = "gold"
                    }
                }
            };
            runner.Initialize(scenario);

            GameState state = new GameState { Resources = new Dictionary<string, int> { { "gold", 5000 } } };
            runner.CheckVictoryConditions(state).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_CheckDefeat_CommandCenterDestroyed()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "defeat-test",
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
                }
            };
            runner.Initialize(scenario);

            GameState alive = new GameState { CommandCenterAlive = true };
            runner.CheckDefeatConditions(alive).Should().BeFalse();

            GameState destroyed = new GameState { CommandCenterAlive = false };
            runner.CheckDefeatConditions(destroyed).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_CheckDefeat_PopulationZero()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "pop-defeat-test",
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition { ConditionType = DefeatConditionType.PopulationZero }
                }
            };
            runner.Initialize(scenario);

            GameState state = new GameState { Population = 0 };
            runner.CheckDefeatConditions(state).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_CheckDefeat_NoConditions_ReturnsFalse()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition { Id = "no-defeat" };
            runner.Initialize(scenario);

            GameState state = new GameState { CommandCenterAlive = false };
            runner.CheckDefeatConditions(state).Should().BeFalse();
        }

        [Fact]
        public void ScenarioRunner_GetPendingEvents_FiresOnce()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "events-test",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "wave3-event",
                        TriggerType = TriggerType.OnWave,
                        TriggerValue = 3,
                        Actions = new List<EventAction>
                        {
                            new EventAction { ActionType = ActionType.ShowMessage }
                        }
                    }
                }
            };
            runner.Initialize(scenario);

            GameState state = new GameState { CurrentWave = 3 };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(state);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("wave3-event");

            // Should not fire again
            IReadOnlyList<ScriptedEvent> eventsAgain = runner.GetPendingEvents(state);
            eventsAgain.Should().BeEmpty();
        }

        [Fact]
        public void ScenarioRunner_ResetEvents_AllowsRefire()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "reset-test",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnWave, TriggerValue = 1 }
                }
            };
            runner.Initialize(scenario);

            GameState state = new GameState { CurrentWave = 1 };
            runner.GetPendingEvents(state).Should().HaveCount(1);
            runner.GetPendingEvents(state).Should().BeEmpty();

            runner.ResetEvents();
            runner.GetPendingEvents(state).Should().HaveCount(1);
        }

        // ── ScenarioPlugin ──────────────────────────────────

        [Fact]
        public void ScenarioPlugin_Constructor_InitializesSubsystems()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            plugin.Runner.Should().NotBeNull();
            plugin.Validator.Should().NotBeNull();
            plugin.DifficultyScaler.Should().NotBeNull();
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_ReturnsValidForEmptyPack()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            ScenarioValidationResult result = plugin.ValidatePack("test-pack", new List<ScenarioDefinition>());
            result.IsValid.Should().BeTrue();
            result.ScenarioCount.Should().Be(0);
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_WarnsAboutMissingConditions()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            List<ScenarioDefinition> scenarios = new List<ScenarioDefinition>
            {
                new ScenarioDefinition { Id = "no-conditions", DisplayName = "No Conditions Test", WaveCount = 5 }
            };

            ScenarioValidationResult result = plugin.ValidatePack("test-pack", scenarios);
            result.IsValid.Should().BeTrue(); // warnings don't make it invalid
            result.Warnings.Should().Contain(w => w.Contains("no victory conditions"));
            result.Warnings.Should().Contain(w => w.Contains("no defeat conditions"));
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_RejectsEmptyPackId()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            Action act = () => plugin.ValidatePack("", new List<ScenarioDefinition>());
            act.Should().Throw<ArgumentException>();
        }

        // ── VictoryCondition Types ──────────────────────────────

        [Fact]
        public void ScenarioRunner_Victory_DestroyTarget()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "destroy-target",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition
                    {
                        ConditionType = VictoryConditionType.DestroyTarget,
                        TargetId = "boss-building"
                    }
                }
            };
            runner.Initialize(scenario);

            // Target not destroyed yet
            GameState state1 = new GameState { BuildingsBuilt = new HashSet<string> { "boss-building" } };
            runner.CheckVictoryConditions(state1).Should().BeFalse();

            // Target destroyed (removed from BuildingsBuilt)
            GameState state2 = new GameState { BuildingsBuilt = new HashSet<string>() };
            runner.CheckVictoryConditions(state2).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_Victory_TimeSurvival()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "time-survival",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition
                    {
                        ConditionType = VictoryConditionType.TimeSurvival,
                        TargetValue = 300
                    }
                }
            };
            runner.Initialize(scenario);

            // Not enough time elapsed
            GameState earlyState = new GameState { ElapsedSeconds = 200 };
            runner.CheckVictoryConditions(earlyState).Should().BeFalse();

            // Target time reached
            GameState lateState = new GameState { ElapsedSeconds = 300 };
            runner.CheckVictoryConditions(lateState).Should().BeTrue();

            // Exceeded target time
            GameState veryLateState = new GameState { ElapsedSeconds = 400 };
            runner.CheckVictoryConditions(veryLateState).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_Victory_Custom()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "custom-victory",
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.Custom }
                }
            };
            runner.Initialize(scenario);

            // Custom conditions never auto-trigger
            GameState state = new GameState();
            runner.CheckVictoryConditions(state).Should().BeFalse();
        }

        // ── DefeatCondition Types ───────────────────────────────

        [Fact]
        public void ScenarioRunner_Defeat_TimeExpired()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "time-expired",
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition
                    {
                        ConditionType = DefeatConditionType.TimeExpired,
                        TargetValue = 600
                    }
                }
            };
            runner.Initialize(scenario);

            // Time not expired
            GameState earlyState = new GameState { ElapsedSeconds = 500 };
            runner.CheckDefeatConditions(earlyState).Should().BeFalse();

            // Time expired
            GameState expiredState = new GameState { ElapsedSeconds = 600 };
            runner.CheckDefeatConditions(expiredState).Should().BeTrue();

            // Time far exceeded
            GameState overState = new GameState { ElapsedSeconds = 1000 };
            runner.CheckDefeatConditions(overState).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_Defeat_ResourceDepleted()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "resource-depleted",
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition { ConditionType = DefeatConditionType.ResourceDepleted }
                }
            };
            runner.Initialize(scenario);

            // All resources at zero
            GameState depletedState = new GameState
            {
                Resources = new Dictionary<string, int>
                {
                    { "food", 0 },
                    { "wood", 0 },
                    { "stone", 0 }
                }
            };
            runner.CheckDefeatConditions(depletedState).Should().BeTrue();

            // At least one resource available
            GameState resourceState = new GameState
            {
                Resources = new Dictionary<string, int>
                {
                    { "food", 10 },
                    { "wood", 0 }
                }
            };
            runner.CheckDefeatConditions(resourceState).Should().BeFalse();

            // Empty resources dict also triggers defeat
            GameState emptyState = new GameState { Resources = new Dictionary<string, int>() };
            runner.CheckDefeatConditions(emptyState).Should().BeTrue();
        }

        [Fact]
        public void ScenarioRunner_Defeat_Custom()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "custom-defeat",
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition { ConditionType = DefeatConditionType.Custom }
                }
            };
            runner.Initialize(scenario);

            // Custom defeat never auto-triggers
            GameState state = new GameState();
            runner.CheckDefeatConditions(state).Should().BeFalse();
        }

        // ── ScriptedEvent Triggers ──────────────────────────────

        [Fact]
        public void ScenarioRunner_Event_OnTime()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "event-time",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "time-event",
                        TriggerType = TriggerType.OnTime,
                        TriggerValue = 120,
                        Actions = new List<EventAction> { new EventAction { ActionType = ActionType.ShowMessage } }
                    }
                }
            };
            runner.Initialize(scenario);

            // Time not reached
            GameState earlyState = new GameState { ElapsedSeconds = 100 };
            runner.GetPendingEvents(earlyState).Should().BeEmpty();

            // Time reached
            GameState timeState = new GameState { ElapsedSeconds = 120 };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(timeState);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("time-event");
        }

        [Fact]
        public void ScenarioRunner_Event_OnPopulation()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "event-pop",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "pop-event",
                        TriggerType = TriggerType.OnPopulation,
                        TriggerValue = 50,
                        Actions = new List<EventAction> { new EventAction { ActionType = ActionType.ShowMessage } }
                    }
                }
            };
            runner.Initialize(scenario);

            // Population not reached
            GameState lowPopState = new GameState { Population = 30 };
            runner.GetPendingEvents(lowPopState).Should().BeEmpty();

            // Population threshold met
            GameState popState = new GameState { Population = 50 };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(popState);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("pop-event");
        }

        [Fact]
        public void ScenarioRunner_Event_OnResource()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "event-resource",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "gold-event",
                        TriggerType = TriggerType.OnResource,
                        TriggerValue = 1000,
                        TriggerTarget = "gold",
                        Actions = new List<EventAction> { new EventAction { ActionType = ActionType.ShowMessage } }
                    }
                }
            };
            runner.Initialize(scenario);

            // Insufficient resource
            GameState lowResState = new GameState
            {
                Resources = new Dictionary<string, int> { { "gold", 500 } }
            };
            runner.GetPendingEvents(lowResState).Should().BeEmpty();

            // Resource threshold met
            GameState resState = new GameState
            {
                Resources = new Dictionary<string, int> { { "gold", 1000 } }
            };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(resState);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("gold-event");
        }

        [Fact]
        public void ScenarioRunner_Event_OnBuildingBuilt()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "event-building",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "barracks-event",
                        TriggerType = TriggerType.OnBuildingBuilt,
                        TriggerTarget = "barracks",
                        Actions = new List<EventAction> { new EventAction { ActionType = ActionType.ShowMessage } }
                    }
                }
            };
            runner.Initialize(scenario);

            // Building not built
            GameState noBuildState = new GameState { BuildingsBuilt = new HashSet<string>() };
            runner.GetPendingEvents(noBuildState).Should().BeEmpty();

            // Building constructed
            GameState buildState = new GameState { BuildingsBuilt = new HashSet<string> { "barracks" } };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(buildState);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("barracks-event");
        }

        [Fact]
        public void ScenarioRunner_Event_OnUnitKilled()
        {
            ScenarioRunner runner = new ScenarioRunner();
            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "event-kills",
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "kill-event",
                        TriggerType = TriggerType.OnUnitKilled,
                        TriggerValue = 25,
                        Actions = new List<EventAction> { new EventAction { ActionType = ActionType.ShowMessage } }
                    }
                }
            };
            runner.Initialize(scenario);

            // Insufficient kills
            GameState fewKillsState = new GameState { UnitsKilled = 10 };
            runner.GetPendingEvents(fewKillsState).Should().BeEmpty();

            // Kill threshold met
            GameState killState = new GameState { UnitsKilled = 25 };
            IReadOnlyList<ScriptedEvent> events = runner.GetPendingEvents(killState);
            events.Should().HaveCount(1);
            events[0].Id.Should().Be("kill-event");
        }

        // ── ScenarioValidator Tests ─────────────────────────────

        [Fact]
        public void ScenarioValidator_ValidScenario()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition validScenario = new ScenarioDefinition
            {
                Id = "valid-scenario",
                DisplayName = "Valid Test Scenario",
                WaveCount = 10,
                StartingResources = new ResourceCost { Food = 100, Wood = 50, Stone = 30, Iron = 20, Gold = 10 },
                AllowedFactions = new List<string>(),
                VictoryConditions = new List<VictoryCondition>
                {
                    new VictoryCondition { ConditionType = VictoryConditionType.SurviveWaves, TargetValue = 10 }
                },
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition { ConditionType = DefeatConditionType.CommandCenterDestroyed }
                },
                ScriptedEvents = new List<ScriptedEvent>()
            };

            IReadOnlyList<string> errors = validator.Validate(validScenario);
            errors.Should().BeEmpty();
        }

        [Fact]
        public void ScenarioValidator_MissingId()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "",
                DisplayName = "Test",
                WaveCount = 5
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("Scenario ID is required"));
        }

        [Fact]
        public void ScenarioValidator_MissingDisplayName()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "test-scenario",
                DisplayName = "",
                WaveCount = 5
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("missing a display name"));
        }

        [Fact]
        public void ScenarioValidator_ZeroWaveCount()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "zero-waves",
                DisplayName = "Zero Waves Test",
                WaveCount = 0
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("invalid wave count"));
        }

        // ── ScenarioPlugin missing-branch coverage ──────────────────────────

        [Fact]
        public void ScenarioPlugin_ValidatePack_NullScenarios_Throws()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            System.Action act = () => plugin.ValidatePack("test-pack", null!);
            act.Should().Throw<System.ArgumentNullException>();
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_MaxDuration_WithNoTimeCondition_AddsWarning()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            List<ScenarioDefinition> scenarios = new List<ScenarioDefinition>
            {
                new ScenarioDefinition
                {
                    Id = "timed-scenario",
                    DisplayName = "Timed",
                    WaveCount = 3,
                    MaxDuration = 600, // duration set, but no time-based conditions
                    VictoryConditions = new List<VictoryCondition>
                    {
                        new VictoryCondition { ConditionType = VictoryConditionType.DestroyTarget, TargetId = "boss" }
                    },
                    DefeatConditions = new List<DefeatCondition>
                    {
                        new DefeatCondition { ConditionType = DefeatConditionType.ResourceDepleted }
                    }
                }
            };

            ScenarioValidationResult result = plugin.ValidatePack("test-pack", scenarios);
            result.Warnings.Should().Contain(w => w.Contains("time-based"));
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_MaxDuration_WithTimeSurvivalVictory_NoWarning()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            List<ScenarioDefinition> scenarios = new List<ScenarioDefinition>
            {
                new ScenarioDefinition
                {
                    Id = "survival-scenario",
                    DisplayName = "Survival",
                    WaveCount = 3,
                    MaxDuration = 600,
                    VictoryConditions = new List<VictoryCondition>
                    {
                        new VictoryCondition { ConditionType = VictoryConditionType.TimeSurvival, TargetValue = 600 }
                    },
                    DefeatConditions = new List<DefeatCondition>
                    {
                        new DefeatCondition { ConditionType = DefeatConditionType.ResourceDepleted }
                    }
                }
            };

            ScenarioValidationResult result = plugin.ValidatePack("test-pack", scenarios);
            result.Warnings.Should().NotContain(w => w.Contains("time-based"));
        }

        [Fact]
        public void ScenarioPlugin_ValidatePack_MaxDuration_WithTimeExpiredDefeat_NoWarning()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioPlugin plugin = new ScenarioPlugin(registries);

            List<ScenarioDefinition> scenarios = new List<ScenarioDefinition>
            {
                new ScenarioDefinition
                {
                    Id = "expiry-scenario",
                    DisplayName = "Time Expiry",
                    WaveCount = 3,
                    MaxDuration = 600,
                    VictoryConditions = new List<VictoryCondition>
                    {
                        new VictoryCondition { ConditionType = VictoryConditionType.DestroyTarget, TargetId = "boss" }
                    },
                    DefeatConditions = new List<DefeatCondition>
                    {
                        new DefeatCondition { ConditionType = DefeatConditionType.TimeExpired, TargetValue = 600 }
                    }
                }
            };

            ScenarioValidationResult result = plugin.ValidatePack("test-pack", scenarios);
            result.Warnings.Should().NotContain(w => w.Contains("time-based"));
        }

        // ── ScenarioValidator additional path coverage ───────────────────────

        [Fact]
        public void ScenarioValidator_NegativeResources_ReportsErrors()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "bad-resources",
                DisplayName = "Bad Resources",
                WaveCount = 3,
                StartingResources = new DINOForge.SDK.Models.ResourceCost
                {
                    Food = -10,
                    Wood = -5,
                    Stone = -1,
                    Iron = -2,
                    Gold = -100
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("negative starting food"));
            errors.Should().Contain(e => e.Contains("negative starting wood"));
            errors.Should().Contain(e => e.Contains("negative starting stone"));
            errors.Should().Contain(e => e.Contains("negative starting iron"));
            errors.Should().Contain(e => e.Contains("negative starting gold"));
        }

        [Fact]
        public void ScenarioValidator_OnWave_OutOfRange_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "wave-event",
                DisplayName = "Wave Event",
                WaveCount = 5,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "evt1",
                        TriggerType = TriggerType.OnWave,
                        TriggerValue = 99 // beyond WaveCount=5
                    }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("OnWave trigger value"));
        }

        [Fact]
        public void ScenarioValidator_OnTime_NegativeValue_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "time-event",
                DisplayName = "Time Event",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnTime, TriggerValue = 0 }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("OnTime trigger with non-positive value"));
        }

        [Fact]
        public void ScenarioValidator_OnResource_NoTarget_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "resource-event",
                DisplayName = "Resource Event",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnResource, TriggerValue = 100, TriggerTarget = null }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("OnResource trigger with no target resource"));
        }

        [Fact]
        public void ScenarioValidator_OnBuildingBuilt_NoTarget_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "building-event",
                DisplayName = "Building Event",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnBuildingBuilt, TriggerValue = 1, TriggerTarget = null }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("OnBuildingBuilt trigger with no target building"));
        }

        [Fact]
        public void ScenarioValidator_DuplicateEventId_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "dup-events",
                DisplayName = "Duplicate Events",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnPopulation, TriggerValue = 50 },
                    new ScriptedEvent { Id = "evt1", TriggerType = TriggerType.OnUnitKilled, TriggerValue = 100 }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("duplicate scripted event ID"));
        }

        [Fact]
        public void ScenarioValidator_SpawnUnits_MissingParameters_ReportsErrors()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "spawn-event",
                DisplayName = "Spawn Event",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "evt1",
                        TriggerType = TriggerType.OnPopulation,
                        TriggerValue = 100,
                        Actions = new List<EventAction>
                        {
                            new EventAction { ActionType = ActionType.SpawnUnits, Parameters = new Dictionary<string, string>() }
                        }
                    }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("SpawnUnits action is missing 'unit_id'"));
            errors.Should().Contain(e => e.Contains("SpawnUnits action is missing 'count'"));
        }

        [Fact]
        public void ScenarioValidator_EnableBuilding_MissingBuildingId_ReportsError()
        {
            RegistryManager registries = new RegistryManager();
            ScenarioValidator validator = new ScenarioValidator(registries);

            ScenarioDefinition scenario = new ScenarioDefinition
            {
                Id = "building-action",
                DisplayName = "Building Action",
                WaveCount = 3,
                ScriptedEvents = new List<ScriptedEvent>
                {
                    new ScriptedEvent
                    {
                        Id = "evt1",
                        TriggerType = TriggerType.OnPopulation,
                        TriggerValue = 50,
                        Actions = new List<EventAction>
                        {
                            new EventAction { ActionType = ActionType.EnableBuilding, Parameters = new Dictionary<string, string>() }
                        }
                    }
                }
            };

            IReadOnlyList<string> errors = validator.Validate(scenario);
            errors.Should().Contain(e => e.Contains("EnableBuilding action is missing 'building_id'"));
        }
    }
}
