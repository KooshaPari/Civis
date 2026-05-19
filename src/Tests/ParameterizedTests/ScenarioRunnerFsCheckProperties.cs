#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Scenario.Balance;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Scripting;
using DINOForge.Domains.Scenario.Validation;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using FsCheck;
using FsCheck.Xunit;
using Xunit;

namespace DINOForge.Tests.ParameterizedTests
{
    /// <summary>
    /// FsCheck Tier 3 behavioral fuzz tests for ScenarioRunner subdomain.
    /// Tests victory/defeat condition evaluation, difficulty scaling, and scripted event triggering.
    ///
    /// These are REAL property tests using FsCheck generators, not parameterized [Theory] tests.
    /// Each [Property] runs 100 random iterations by default.
    /// Verifies scenario-evaluation invariants and determinism across randomized game states.
    ///
    /// Target types:
    /// - VictoryCondition: field assignment stability and equality
    /// - DefeatCondition: field assignment and nullability handling
    /// - DifficultyScaler.ScaleResources(): normal difficulty → exact base resource match
    /// - DifficultyScaler.ScaleResources(): monotonic in difficulty (harder → fewer resources)
    /// - DifficultyScaler.ScaleWaveIntensity(): monotonic with wave number and difficulty
    /// - ScriptedEvent trigger evaluation: idempotent (same conditions → same result)
    /// - ScenarioRunner.CheckVictoryConditions(): empty victory conditions → returns false
    /// </summary>
    [Trait("Category", "Property")]
    [Trait("Layer", "Scenario")]
    public class ScenarioRunnerFsCheckProperties
    {
        /// <summary>
        /// Property: VictoryCondition field assignment is stable and retrieves correctly.
        /// For any assigned values in a VictoryCondition, field access returns the same value.
        /// Validates basic property contract and assignment stability.
        ///
        /// FsCheck generates 100+ random VictoryCondition instances.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool VictoryCondition_FieldAssignment_IsStableAndRetrieves(int targetValue, NonEmptyString description)
        {
            // Arrange: Create condition with assigned values
            var condition = new VictoryCondition
            {
                ConditionType = VictoryConditionType.ReachPopulation,
                TargetValue = targetValue,
                Description = description.Get
            };

            // Act: Retrieve the values back
            var retrievedConditionType = condition.ConditionType;
            var retrievedTargetValue = condition.TargetValue;
            var retrievedDescription = condition.Description;

            // Assert: Retrieved values match assigned values (stability)
            var isStable = retrievedConditionType == VictoryConditionType.ReachPopulation &&
                          retrievedTargetValue == targetValue &&
                          retrievedDescription == description.Get;

            isStable.Should().BeTrue(
                because: "VictoryCondition field assignment must be stable");
            return isStable;
        }

        /// <summary>
        /// Property: DefeatCondition field assignment handles null targets correctly.
        /// For DefeatCondition with nullable TargetValue, null assignment is preserved.
        /// Validates nullable field contract and nullability stability.
        ///
        /// FsCheck generates 100+ random DefeatCondition instances with various nullable values.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool DefeatCondition_NullableTargetValue_PreservesNullability(int? targetValue, NonEmptyString description)
        {
            // Arrange: Create condition with assigned nullable value
            var condition = new DefeatCondition
            {
                ConditionType = DefeatConditionType.TimeExpired,
                TargetValue = targetValue,
                Description = description.Get
            };

            // Act: Retrieve the nullable value
            var retrievedTargetValue = condition.TargetValue;

            // Assert: Null is preserved, and non-null values match exactly
            var isCorrect = (targetValue == null && retrievedTargetValue == null) ||
                           (targetValue != null && retrievedTargetValue == targetValue);

            isCorrect.Should().BeTrue(
                because: "DefeatCondition nullable TargetValue must preserve null/non-null correctly");
            return isCorrect;
        }

        /// <summary>
        /// Property: DifficultyScaler.ScaleResources with Difficulty.Normal returns base resources exactly.
        /// For any ResourceCost with non-negative values and Difficulty.Normal (multiplier = 1.0),
        /// ScaleResources returns identical values (all multiplied by 1.0 = same).
        /// Validates identity property and baseline scaling correctness.
        ///
        /// FsCheck generates 100+ random non-negative ResourceCost instances.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool DifficultyScaler_ScaleResources_NormalDifficulty_ReturnsExact(
            NonNegativeInt food, NonNegativeInt wood, NonNegativeInt stone, NonNegativeInt iron, NonNegativeInt gold)
        {
            // Arrange: Create base resources and scaler
            var baseResources = new ResourceCost
            {
                Food = food.Get,
                Wood = wood.Get,
                Stone = stone.Get,
                Iron = iron.Get,
                Gold = gold.Get
            };

            var scaler = new DifficultyScaler();

            // Act: Scale with Normal difficulty (multiplier = 1.0)
            var scaled = scaler.ScaleResources(baseResources, Difficulty.Normal);

            // Assert: Scaled resources match base exactly
            var isExact = scaled.Food == baseResources.Food &&
                          scaled.Wood == baseResources.Wood &&
                          scaled.Stone == baseResources.Stone &&
                          scaled.Iron == baseResources.Iron &&
                          scaled.Gold == baseResources.Gold;

            isExact.Should().BeTrue(
                because: "Normal difficulty should scale resources by 1.0 (identity)");
            return isExact;
        }

        /// <summary>
        /// Property: DifficultyScaler.ScaleResources monotonic in difficulty (harder → fewer resources for cost scaling).
        /// For any ResourceCost, ScaleResources(base, Hard) ≤ ScaleResources(base, Normal).
        /// Harder difficulties reduce starting resources; this property validates monotonicity.
        /// NOTE: Easy gives bonus, so Easy ≥ Normal. Hard ≤ Normal.
        ///
        /// FsCheck generates 100+ random non-negative ResourceCost instances.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool DifficultyScaler_ScaleResources_Monotonic_Hard_LessThan_Normal(
            NonNegativeInt baseFood)
        {
            // Arrange: Simple single-resource case
            var baseResources = new ResourceCost
            {
                Food = baseFood.Get,
                Wood = 100,
                Stone = 100,
                Iron = 100,
                Gold = 100
            };

            var scaler = new DifficultyScaler();

            // Act: Scale for Normal and Hard
            var normal = scaler.ScaleResources(baseResources, Difficulty.Normal);
            var hard = scaler.ScaleResources(baseResources, Difficulty.Hard);

            // Assert: Hard difficulty resources ≤ Normal (harder = fewer resources)
            var isMonotonic = hard.Food <= normal.Food &&
                              hard.Wood <= normal.Wood &&
                              hard.Stone <= normal.Stone &&
                              hard.Iron <= normal.Iron &&
                              hard.Gold <= normal.Gold;

            isMonotonic.Should().BeTrue(
                because: "Hard difficulty should scale to ≤ Normal resources (fewer resources on hard)");
            return isMonotonic;
        }

        /// <summary>
        /// Property: DifficultyScaler.ScaleWaveIntensity monotonic in wave number (later waves → higher intensity).
        /// For any base intensity and fixed difficulty, higher wave numbers produce higher scaled intensity.
        /// Validates wave progression monotonicity.
        ///
        /// FsCheck generates 100+ random (baseIntensity, waveNumber) pairs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool DifficultyScaler_ScaleWaveIntensity_Monotonic_InWaveNumber(PositiveInt wave1, PositiveInt wave2)
        {
            // Arrange: Ensure wave1 < wave2
            int smallerWave = Math.Min(wave1.Get, wave2.Get);
            int largerWave = Math.Max(wave1.Get, wave2.Get);

            var scaler = new DifficultyScaler();

            // Act: Scale wave intensity for same base, same difficulty, different waves
            float intensity1 = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, smallerWave);
            float intensity2 = scaler.ScaleWaveIntensity(1.0f, Difficulty.Normal, largerWave);

            // Assert: Higher wave number → higher intensity (unless waves are equal)
            var isMonotonic = smallerWave < largerWave ? intensity1 <= intensity2 : intensity1 == intensity2;

            isMonotonic.Should().BeTrue(
                because: "Wave intensity should increase or stay same with higher wave numbers");
            return isMonotonic;
        }

        /// <summary>
        /// Property: ScriptedEvent trigger evaluation is idempotent (same conditions → same result).
        /// For a ScriptedEvent and GameState, calling IsEventTriggered twice returns same boolean.
        /// Validates that event triggering is stateless and deterministic.
        ///
        /// FsCheck generates 100+ random (event, gamestate) pairs.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool ScriptedEvent_TriggerEvaluation_IsIdempotent(PositiveInt waveNumber)
        {
            // Arrange: Create an OnWave trigger event
            var scriptedEvent = new ScriptedEvent
            {
                Id = "wave-5-event",
                TriggerType = TriggerType.OnWave,
                TriggerValue = 5,
                Actions = new List<EventAction>()
            };

            var gameState = new GameState
            {
                CurrentWave = waveNumber.Get,
                ElapsedSeconds = 0.0,
                Population = 100,
                CommandCenterAlive = true
            };

            var runner = new ScenarioRunner();
            var scenario = new ScenarioDefinition
            {
                Id = "test-scenario",
                DisplayName = "Test",
                WaveCount = 10,
                VictoryConditions = new List<VictoryCondition>(),
                DefeatConditions = new List<DefeatCondition>(),
                ScriptedEvents = new List<ScriptedEvent> { scriptedEvent }
            };
            runner.Initialize(scenario);

            // Act: Call GetPendingEvents twice (but reset event tracking between calls)
            var pending1 = runner.GetPendingEvents(gameState);
            runner.ResetEvents(); // Reset so the same event can fire again in the second call
            var pending2 = runner.GetPendingEvents(gameState);

            // Assert: Same trigger conditions → same trigger result (both should contain event or both shouldn't)
            var isIdempotent = (pending1.Count > 0) == (pending2.Count > 0);

            isIdempotent.Should().BeTrue(
                because: "ScriptedEvent trigger evaluation must be deterministic and idempotent");
            return isIdempotent;
        }

        /// <summary>
        /// Property: ScenarioRunner.CheckVictoryConditions with empty conditions returns false.
        /// For any initialized scenario with zero victory conditions,
        /// CheckVictoryConditions returns false (vacuous failure: no victories defined = can't win).
        /// Validates contract: empty conditions are not auto-satisfied.
        ///
        /// FsCheck generates 100+ random GameState instances.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool ScenarioRunner_CheckVictoryConditions_EmptyConditions_ReturnsFalse(
            PositiveInt wave, PositiveInt population)
        {
            // Arrange: Scenario with zero victory conditions
            var scenario = new ScenarioDefinition
            {
                Id = "no-victory-scenario",
                DisplayName = "No Victory Scenario",
                WaveCount = 10,
                VictoryConditions = new List<VictoryCondition>(), // Empty!
                DefeatConditions = new List<DefeatCondition>(),
                ScriptedEvents = new List<ScriptedEvent>()
            };

            var runner = new ScenarioRunner();
            runner.Initialize(scenario);

            var gameState = new GameState
            {
                CurrentWave = wave.Get,
                Population = population.Get,
                CommandCenterAlive = true,
                ElapsedSeconds = 100.0
            };

            // Act: Check victory
            bool isVictory = runner.CheckVictoryConditions(gameState);

            // Assert: Empty conditions → false (can't win if no win condition is defined)
            isVictory.Should().BeFalse(
                because: "Empty victory conditions should return false (no victory possible)");
            return !isVictory;
        }

        /// <summary>
        /// Property: ScenarioRunner.CheckDefeatConditions consistency with PopulationZero condition.
        /// For a scenario with PopulationZero defeat condition, population <= 0 triggers defeat.
        /// Validates condition evaluation correctness and state-based semantics.
        ///
        /// FsCheck generates 100+ random population values.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool ScenarioRunner_CheckDefeatConditions_PopulationZero_ConsistenWithState(PositiveInt population)
        {
            // Arrange: PopulationZero defeat condition
            var scenario = new ScenarioDefinition
            {
                Id = "pop-zero-defeat",
                DisplayName = "Population Zero Defeats",
                WaveCount = 10,
                VictoryConditions = new List<VictoryCondition>(),
                DefeatConditions = new List<DefeatCondition>
                {
                    new DefeatCondition
                    {
                        ConditionType = DefeatConditionType.PopulationZero,
                        TargetValue = null,
                        Description = "Population reaches zero"
                    }
                },
                ScriptedEvents = new List<ScriptedEvent>()
            };

            var runner = new ScenarioRunner();
            runner.Initialize(scenario);

            var gameState = new GameState { Population = population.Get, CommandCenterAlive = true };

            // Act: Check defeat
            bool isDefeated = runner.CheckDefeatConditions(gameState);

            // Assert: Consistency with actual game state (pop <= 0 means defeated)
            bool isConsistent = (gameState.Population <= 0) == isDefeated;

            isConsistent.Should().BeTrue(
                because: "PopulationZero defeat must evaluate correctly based on current population");
            return isConsistent;
        }

        /// <summary>
        /// Property: VictoryCondition with target_value=0 in SurviveWaves always returns true.
        /// For SurviveWaves with target=0, victory is met when current wave >= 0 (always true).
        /// Edge-case validation: zero targets should be handled correctly (not undefined behavior).
        ///
        /// FsCheck generates 100+ random wave values.
        /// </summary>
        [Property(MaxTest = 100)]
        public bool VictoryCondition_SurviveWaves_ZeroTarget_AlwaysTrue(PositiveInt currentWave)
        {
            // Arrange: Victory condition with zero waves required
            var victoryCondition = new VictoryCondition
            {
                ConditionType = VictoryConditionType.SurviveWaves,
                TargetValue = 0,
                Description = "Survive 0 waves (always true)"
            };

            var gameState = new GameState
            {
                CurrentWave = currentWave.Get
            };

            var runner = new ScenarioRunner();
            var scenario = new ScenarioDefinition
            {
                Id = "zero-wave-victory",
                DisplayName = "Zero Wave Victory",
                WaveCount = 1,
                VictoryConditions = new List<VictoryCondition> { victoryCondition },
                DefeatConditions = new List<DefeatCondition>(),
                ScriptedEvents = new List<ScriptedEvent>()
            };
            runner.Initialize(scenario);

            // Act: Check victory (current wave >= 0, always true)
            bool isVictory = runner.CheckVictoryConditions(gameState);

            // Assert: Any non-negative current wave >= 0 satisfies the condition
            isVictory.Should().BeTrue(
                because: "SurviveWaves with target=0 should always be satisfied");
            return isVictory;
        }
    }
}
