using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Scenario.Models;

namespace DINOForge.Domains.Scenario.Scripting
{
    /// <summary>
    /// Evaluates scenario state against victory conditions, defeat conditions, and scripted
    /// event triggers. Tracks which events have already fired to prevent re-triggering.
    /// </summary>
    public sealed class ScenarioRunner
    {
        private ScenarioDefinition? _scenario;
        private readonly HashSet<string> _firedEventIds = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// The scenario definition currently loaded into this runner.
        /// </summary>
        public ScenarioDefinition? CurrentScenario => _scenario;

        /// <summary>
        /// Whether the runner has been initialized with a scenario.
        /// </summary>
        public bool IsInitialized => _scenario != null;

        /// <summary>
        /// Initialize the runner with a scenario definition. Resets all previously fired event tracking.
        /// </summary>
        /// <param name="scenario">The scenario definition to run.</param>
        /// <exception cref="ArgumentNullException">Thrown if <paramref name="scenario"/> is null.</exception>
        public void Initialize(ScenarioDefinition scenario)
        {
            _scenario = scenario ?? throw new ArgumentNullException(nameof(scenario));
            _firedEventIds.Clear();
        }

        /// <summary>
        /// Check whether all victory conditions for the current scenario are met.
        /// Returns false if no victory conditions are defined.
        /// </summary>
        /// <param name="gameState">Current snapshot of game state.</param>
        /// <returns>True if all victory conditions are satisfied.</returns>
        /// <exception cref="InvalidOperationException">Thrown if the runner has not been initialized.</exception>
        public bool CheckVictoryConditions(GameState gameState)
        {
            if (_scenario == null) throw new InvalidOperationException("ScenarioRunner has not been initialized. Call Initialize() first.");
            if (gameState == null) throw new ArgumentNullException(nameof(gameState));

            List<VictoryCondition> conditions = _scenario.VictoryConditions;
            if (conditions.Count == 0) return false;

            foreach (VictoryCondition condition in conditions)
            {
                if (!IsVictoryConditionMet(condition, gameState))
                    return false;
            }

            return true;
        }

        /// <summary>
        /// Check whether any defeat condition for the current scenario is met.
        /// Returns false if no defeat conditions are defined.
        /// </summary>
        /// <param name="gameState">Current snapshot of game state.</param>
        /// <returns>True if any defeat condition is satisfied.</returns>
        /// <exception cref="InvalidOperationException">Thrown if the runner has not been initialized.</exception>
        public bool CheckDefeatConditions(GameState gameState)
        {
            if (_scenario == null) throw new InvalidOperationException("ScenarioRunner has not been initialized. Call Initialize() first.");
            if (gameState == null) throw new ArgumentNullException(nameof(gameState));

            List<DefeatCondition> conditions = _scenario.DefeatConditions;
            if (conditions.Count == 0) return false;

            foreach (DefeatCondition condition in conditions)
            {
                if (IsDefeatConditionMet(condition, gameState))
                    return true;
            }

            return false;
        }

        /// <summary>
        /// Get all scripted events whose trigger conditions are met by the current game state
        /// and that have not already been fired. Returned events are marked as fired and will
        /// not be returned again.
        /// </summary>
        /// <param name="gameState">Current snapshot of game state.</param>
        /// <returns>List of newly triggered scripted events.</returns>
        /// <exception cref="InvalidOperationException">Thrown if the runner has not been initialized.</exception>
        public IReadOnlyList<ScriptedEvent> GetPendingEvents(GameState gameState)
        {
            if (_scenario == null) throw new InvalidOperationException("ScenarioRunner has not been initialized. Call Initialize() first.");
            if (gameState == null) throw new ArgumentNullException(nameof(gameState));

            List<ScriptedEvent> pending = new List<ScriptedEvent>();

            foreach (ScriptedEvent scriptedEvent in _scenario.ScriptedEvents)
            {
                if (_firedEventIds.Contains(scriptedEvent.Id))
                    continue;

                if (IsEventTriggered(scriptedEvent, gameState))
                {
                    _firedEventIds.Add(scriptedEvent.Id);
                    pending.Add(scriptedEvent);
                }
            }

            return pending.AsReadOnly();
        }

        /// <summary>
        /// Reset the fired event tracking without changing the loaded scenario.
        /// Useful for restarting a scenario.
        /// </summary>
        public void ResetEvents()
        {
            _firedEventIds.Clear();
        }

        private bool IsVictoryConditionMet(VictoryCondition condition, GameState gameState)
        {
            switch (condition.ConditionType)
            {
                case VictoryConditionType.SurviveWaves:
                    return gameState.CurrentWave >= condition.TargetValue;

                case VictoryConditionType.DestroyTarget:
                    // DestroyTarget checks GameState.DestroyedEntities (populated by ECS callers
                    // on the Runtime side) — falls back to BuildingsBuilt for backward compat
                    // with callers that only track construction. The Scenario domain does not
                    // query ECS directly; Runtime-side bridges populate Live/DestroyedEntities.
                    if (string.IsNullOrEmpty(condition.TargetId)) return false;
                    string destroyTargetId = condition.TargetId!;
                    // Primary path: explicit destruction observed by ECS bridge.
                    if (gameState.DestroyedEntities.Contains(destroyTargetId)) return true;
                    // Secondary path: target was alive (tracked) but is no longer live.
                    if (gameState.LiveEntities.Count > 0)
                    {
                        if (!gameState.LiveEntities.Contains(destroyTargetId)
                            && (gameState.BuildingsBuilt.Contains(destroyTargetId)
                                || gameState.DestroyedEntities.Count > 0))
                        {
                            return true;
                        }
                    }
                    // Legacy fallback: target absent from BuildingsBuilt (pre-ECS callers).
                    return !gameState.BuildingsBuilt.Contains(destroyTargetId);

                case VictoryConditionType.ReachPopulation:
                    return gameState.Population >= condition.TargetValue;

                case VictoryConditionType.AccumulateResource:
                    if (string.IsNullOrEmpty(condition.TargetId)) return false;
                    return gameState.Resources.TryGetValue(condition.TargetId!, out int amount)
                           && amount >= condition.TargetValue;

                case VictoryConditionType.TimeSurvival:
                    return gameState.ElapsedSeconds >= condition.TargetValue;

                case VictoryConditionType.Custom:
                    // Custom conditions are evaluated externally; always return false here.
                    return false;

                default:
                    return false;
            }
        }

        private bool IsDefeatConditionMet(DefeatCondition condition, GameState gameState)
        {
            switch (condition.ConditionType)
            {
                case DefeatConditionType.CommandCenterDestroyed:
                    return !gameState.CommandCenterAlive;

                case DefeatConditionType.PopulationZero:
                    return gameState.Population <= 0;

                case DefeatConditionType.TimeExpired:
                    int timeLimit = condition.TargetValue ?? 0;
                    return timeLimit > 0 && gameState.ElapsedSeconds >= timeLimit;

                case DefeatConditionType.ResourceDepleted:
                    // Check if all resources are at zero
                    if (gameState.Resources.Count == 0) return true;
                    foreach (KeyValuePair<string, int> kvp in gameState.Resources)
                    {
                        if (kvp.Value > 0) return false;
                    }
                    return true;

                case DefeatConditionType.Custom:
                    return false;

                default:
                    return false;
            }
        }

        private bool IsEventTriggered(ScriptedEvent scriptedEvent, GameState gameState)
        {
            switch (scriptedEvent.TriggerType)
            {
                case TriggerType.OnWave:
                    return gameState.CurrentWave >= scriptedEvent.TriggerValue;

                case TriggerType.OnTime:
                    return gameState.ElapsedSeconds >= scriptedEvent.TriggerValue;

                case TriggerType.OnPopulation:
                    return gameState.Population >= scriptedEvent.TriggerValue;

                case TriggerType.OnResource:
                    if (string.IsNullOrEmpty(scriptedEvent.TriggerTarget)) return false;
                    return gameState.Resources.TryGetValue(scriptedEvent.TriggerTarget!, out int resourceAmount)
                           && resourceAmount >= scriptedEvent.TriggerValue;

                case TriggerType.OnBuildingBuilt:
                    if (string.IsNullOrEmpty(scriptedEvent.TriggerTarget)) return false;
                    return gameState.BuildingsBuilt.Contains(scriptedEvent.TriggerTarget!);

                case TriggerType.OnUnitKilled:
                    return gameState.UnitsKilled >= scriptedEvent.TriggerValue;

                default:
                    return false;
            }
        }
    }
}
