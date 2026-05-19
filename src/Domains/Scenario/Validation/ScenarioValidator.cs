using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Models;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Scenario.Validation
{
    /// <summary>
    /// Validates scenario definitions for correctness and consistency.
    /// Checks faction references, condition validity, scripted event integrity,
    /// and resource constraints.
    /// </summary>
    public sealed class ScenarioValidator
    {
        private readonly RegistryManager _registries;

        /// <summary>
        /// Create a new scenario validator with access to the loaded registries.
        /// </summary>
        /// <param name="registries">The registry manager containing all loaded content.</param>
        public ScenarioValidator(RegistryManager registries)
        {
            _registries = registries ?? throw new ArgumentNullException(nameof(registries));
        }

        /// <summary>
        /// Validate a scenario definition against all validation rules.
        /// </summary>
        /// <param name="scenario">The scenario definition to validate.</param>
        /// <returns>List of validation error messages. Empty list means valid.</returns>
        public IReadOnlyList<string> Validate(ScenarioDefinition scenario)
        {
            if (scenario == null) throw new ArgumentNullException(nameof(scenario));

            List<string> errors = new List<string>();

            ValidateBasicFields(scenario, errors);
            ValidateWaveCount(scenario, errors);
            ValidateStartingResources(scenario, errors);
            ValidateAllowedFactions(scenario, errors);
            ValidateVictoryConditions(scenario, errors);
            ValidateDefeatConditions(scenario, errors);
            ValidateScriptedEvents(scenario, errors);

            return errors.AsReadOnly();
        }

        private void ValidateBasicFields(ScenarioDefinition scenario, List<string> errors)
        {
            if (string.IsNullOrWhiteSpace(scenario.Id))
            {
                errors.Add("Scenario ID is required.");
            }

            if (string.IsNullOrWhiteSpace(scenario.DisplayName))
            {
                errors.Add($"Scenario '{scenario.Id}' is missing a display name.");
            }
        }

        private void ValidateWaveCount(ScenarioDefinition scenario, List<string> errors)
        {
            if (scenario.WaveCount <= 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has invalid wave count {scenario.WaveCount}. Must be positive.");
            }
        }

        private void ValidateStartingResources(ScenarioDefinition scenario, List<string> errors)
        {
            if (scenario.StartingResources.Food < 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has negative starting food ({scenario.StartingResources.Food}).");
            }
            if (scenario.StartingResources.Wood < 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has negative starting wood ({scenario.StartingResources.Wood}).");
            }
            if (scenario.StartingResources.Stone < 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has negative starting stone ({scenario.StartingResources.Stone}).");
            }
            if (scenario.StartingResources.Iron < 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has negative starting iron ({scenario.StartingResources.Iron}).");
            }
            if (scenario.StartingResources.Gold < 0)
            {
                errors.Add($"Scenario '{scenario.Id}' has negative starting gold ({scenario.StartingResources.Gold}).");
            }
        }

        private void ValidateAllowedFactions(ScenarioDefinition scenario, List<string> errors)
        {
            foreach (string factionId in scenario.AllowedFactions)
            {
                if (!_registries.Factions.Contains(factionId))
                {
                    errors.Add($"Scenario '{scenario.Id}' references unknown faction '{factionId}'.");
                }
            }
        }

        private void ValidateVictoryConditions(ScenarioDefinition scenario, List<string> errors)
        {
            foreach (VictoryCondition condition in scenario.VictoryConditions)
            {
                switch (condition.ConditionType)
                {
                    case VictoryConditionType.SurviveWaves:
                        if (condition.TargetValue <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition SurviveWaves has invalid target value {condition.TargetValue}. Must be positive.");
                        }
                        break;

                    case VictoryConditionType.DestroyTarget:
                        if (string.IsNullOrWhiteSpace(condition.TargetId))
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition DestroyTarget is missing a target ID.");
                        }
                        break;

                    case VictoryConditionType.ReachPopulation:
                        if (condition.TargetValue <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition ReachPopulation has invalid target value {condition.TargetValue}. Must be positive.");
                        }
                        break;

                    case VictoryConditionType.AccumulateResource:
                        if (string.IsNullOrWhiteSpace(condition.TargetId))
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition AccumulateResource is missing a target resource ID.");
                        }
                        if (condition.TargetValue <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition AccumulateResource has invalid target value {condition.TargetValue}. Must be positive.");
                        }
                        break;

                    case VictoryConditionType.TimeSurvival:
                        if (condition.TargetValue <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' victory condition TimeSurvival has invalid target value {condition.TargetValue}. Must be positive.");
                        }
                        break;

                    case VictoryConditionType.Custom:
                        // Custom conditions have no structural constraints
                        break;
                }
            }
        }

        private void ValidateDefeatConditions(ScenarioDefinition scenario, List<string> errors)
        {
            foreach (DefeatCondition condition in scenario.DefeatConditions)
            {
                switch (condition.ConditionType)
                {
                    case DefeatConditionType.TimeExpired:
                        if (condition.TargetValue.HasValue && condition.TargetValue.Value <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' defeat condition TimeExpired has invalid target value {condition.TargetValue.Value}. Must be positive.");
                        }
                        break;

                    case DefeatConditionType.CommandCenterDestroyed:
                    case DefeatConditionType.PopulationZero:
                    case DefeatConditionType.ResourceDepleted:
                    case DefeatConditionType.Custom:
                        // These conditions have no additional structural constraints
                        break;
                }
            }
        }

        private void ValidateScriptedEvents(ScenarioDefinition scenario, List<string> errors)
        {
            HashSet<string> eventIds = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

            foreach (ScriptedEvent scriptedEvent in scenario.ScriptedEvents)
            {
                if (string.IsNullOrWhiteSpace(scriptedEvent.Id))
                {
                    errors.Add($"Scenario '{scenario.Id}' contains a scripted event with no ID.");
                    continue;
                }

                if (!eventIds.Add(scriptedEvent.Id))
                {
                    errors.Add($"Scenario '{scenario.Id}' contains duplicate scripted event ID '{scriptedEvent.Id}'.");
                }

                // Validate trigger-specific constraints
                switch (scriptedEvent.TriggerType)
                {
                    case TriggerType.OnWave:
                        if (scriptedEvent.TriggerValue <= 0 || scriptedEvent.TriggerValue > scenario.WaveCount)
                        {
                            errors.Add($"Scenario '{scenario.Id}' scripted event '{scriptedEvent.Id}' has OnWave trigger value {scriptedEvent.TriggerValue} outside valid range [1, {scenario.WaveCount}].");
                        }
                        break;

                    case TriggerType.OnTime:
                        if (scriptedEvent.TriggerValue <= 0)
                        {
                            errors.Add($"Scenario '{scenario.Id}' scripted event '{scriptedEvent.Id}' has OnTime trigger with non-positive value.");
                        }
                        break;

                    case TriggerType.OnResource:
                        if (string.IsNullOrWhiteSpace(scriptedEvent.TriggerTarget))
                        {
                            errors.Add($"Scenario '{scenario.Id}' scripted event '{scriptedEvent.Id}' has OnResource trigger with no target resource specified.");
                        }
                        break;

                    case TriggerType.OnBuildingBuilt:
                        if (string.IsNullOrWhiteSpace(scriptedEvent.TriggerTarget))
                        {
                            errors.Add($"Scenario '{scenario.Id}' scripted event '{scriptedEvent.Id}' has OnBuildingBuilt trigger with no target building specified.");
                        }
                        break;
                }

                // Validate actions
                foreach (EventAction action in scriptedEvent.Actions)
                {
                    ValidateEventAction(scenario.Id, scriptedEvent.Id, action, errors);
                }
            }
        }

        private void ValidateEventAction(string scenarioId, string eventId, EventAction action, List<string> errors)
        {
            switch (action.ActionType)
            {
                case ActionType.SpawnUnits:
                    if (!action.Parameters.ContainsKey("unit_id"))
                    {
                        errors.Add($"Scenario '{scenarioId}' event '{eventId}' SpawnUnits action is missing 'unit_id' parameter.");
                    }
                    if (!action.Parameters.ContainsKey("count"))
                    {
                        errors.Add($"Scenario '{scenarioId}' event '{eventId}' SpawnUnits action is missing 'count' parameter.");
                    }
                    else if (action.Parameters.TryGetValue("count", out string? countStr) && int.TryParse(countStr, out int count) && count <= 0)
                    {
                        errors.Add($"Scenario '{scenarioId}' event '{eventId}' SpawnUnits action has invalid count '{countStr}'.");
                    }
                    break;

                case ActionType.EnableBuilding:
                case ActionType.DisableBuilding:
                    if (!action.Parameters.ContainsKey("building_id"))
                    {
                        errors.Add($"Scenario '{scenarioId}' event '{eventId}' {action.ActionType} action is missing 'building_id' parameter.");
                    }
                    break;
            }
        }
    }
}
