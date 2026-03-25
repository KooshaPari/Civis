using System;
using System.Collections.Generic;
using DINOForge.Domains.Scenario.Balance;
using DINOForge.Domains.Scenario.Models;
using DINOForge.Domains.Scenario.Registries;
using DINOForge.Domains.Scenario.Scripting;
using DINOForge.Domains.Scenario.Validation;
using DINOForge.SDK.Registry;

namespace DINOForge.Domains.Scenario
{
    /// <summary>
    /// Entry point for the Scenario domain plugin. Provides access to scenario subsystems:
    /// scenario registry, content loader, runner, validator, and difficulty scaler.
    /// </summary>
    public class ScenarioPlugin
    {
        /// <summary>
        /// Registry of all loaded scenario definitions.
        /// </summary>
        public ScenarioRegistry Scenarios { get; }

        /// <summary>
        /// Content loader for loading scenarios from pack directories.
        /// </summary>
        public ScenarioContentLoader ContentLoader { get; }

        /// <summary>
        /// Runner for evaluating scenario state, checking conditions, and firing scripted events.
        /// </summary>
        public ScenarioRunner Runner { get; }

        /// <summary>
        /// Validator for checking scenario definitions against registries and structural rules.
        /// </summary>
        public ScenarioValidator Validator { get; }

        /// <summary>
        /// Scaler for adjusting scenario parameters based on difficulty level.
        /// </summary>
        public DifficultyScaler DifficultyScaler { get; }

        private readonly RegistryManager _registries;

        /// <summary>
        /// Initialize the scenario plugin with pre-loaded registries.
        /// </summary>
        /// <param name="registries">The registry manager containing all loaded content.</param>
        public ScenarioPlugin(RegistryManager registries)
        {
            _registries = registries ?? throw new ArgumentNullException(nameof(registries));

            Scenarios = new ScenarioRegistry();
            ContentLoader = new ScenarioContentLoader(Scenarios);
            Runner = new ScenarioRunner();
            Validator = new ScenarioValidator(registries);
            DifficultyScaler = new DifficultyScaler();
        }

        /// <summary>
        /// Validate all scenario definitions from a specific pack. Iterates through all registered
        /// scenarios matching the given pack ID and runs full validation on each.
        /// </summary>
        /// <param name="packId">The pack identifier to scope validation to.</param>
        /// <param name="scenarios">The scenarios to validate for this pack.</param>
        /// <returns>A validation result with aggregated errors and warnings.</returns>
        public ScenarioValidationResult ValidatePack(string packId, IEnumerable<ScenarioDefinition> scenarios)
        {
            if (string.IsNullOrWhiteSpace(packId)) throw new ArgumentException("Pack ID is required.", nameof(packId));
            if (scenarios == null) throw new ArgumentNullException(nameof(scenarios));

            List<string> errors = new List<string>();
            List<string> warnings = new List<string>();
            int scenarioCount = 0;

            foreach (ScenarioDefinition scenario in scenarios)
            {
                scenarioCount++;

                IReadOnlyList<string> scenarioErrors = Validator.Validate(scenario);
                foreach (string error in scenarioErrors)
                {
                    errors.Add(error);
                }

                // Warnings for scenarios without victory conditions
                if (scenario.VictoryConditions.Count == 0)
                {
                    warnings.Add($"Scenario '{scenario.Id}' has no victory conditions defined.");
                }

                // Warnings for scenarios without defeat conditions
                if (scenario.DefeatConditions.Count == 0)
                {
                    warnings.Add($"Scenario '{scenario.Id}' has no defeat conditions defined.");
                }

                // Warning if max duration is set but no TimeSurvival victory or TimeExpired defeat
                if (scenario.MaxDuration > 0)
                {
                    bool hasTimeCondition = false;
                    foreach (VictoryCondition vc in scenario.VictoryConditions)
                    {
                        if (vc.ConditionType == VictoryConditionType.TimeSurvival)
                        {
                            hasTimeCondition = true;
                            break;
                        }
                    }
                    if (!hasTimeCondition)
                    {
                        foreach (DefeatCondition dc in scenario.DefeatConditions)
                        {
                            if (dc.ConditionType == DefeatConditionType.TimeExpired)
                            {
                                hasTimeCondition = true;
                                break;
                            }
                        }
                    }
                    if (!hasTimeCondition)
                    {
                        warnings.Add($"Scenario '{scenario.Id}' has max_duration set but no time-based victory or defeat condition.");
                    }
                }
            }

            return new ScenarioValidationResult(
                packId: packId,
                isValid: errors.Count == 0,
                errors: errors.AsReadOnly(),
                warnings: warnings.AsReadOnly(),
                scenarioCount: scenarioCount);
        }
    }
}
