using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Scenario.Models;

namespace DINOForge.Domains.Scenario.Registries
{
    /// <summary>
    /// Registry of scenario definitions. Supports custom scenario registration and lookup.
    /// </summary>
    public class ScenarioRegistry
    {
        private readonly Dictionary<string, ScenarioDefinition> _scenarios =
            new Dictionary<string, ScenarioDefinition>(StringComparer.OrdinalIgnoreCase);

        /// <summary>
        /// All registered scenarios.
        /// </summary>
        public IReadOnlyList<ScenarioDefinition> All => _scenarios.Values.ToList().AsReadOnly();

        /// <summary>
        /// Number of registered scenarios.
        /// </summary>
        public int Count => _scenarios.Count;

        /// <summary>
        /// Retrieve a scenario by its identifier.
        /// </summary>
        /// <param name="id">Scenario identifier.</param>
        /// <returns>The matching scenario definition.</returns>
        /// <exception cref="KeyNotFoundException">Thrown when no scenario with the given id exists.</exception>
        public ScenarioDefinition GetScenario(string id)
        {
            if (_scenarios.TryGetValue(id, out ScenarioDefinition? scenario))
                return scenario;

            throw new KeyNotFoundException($"No scenario registered with id '{id}'.");
        }

        /// <summary>
        /// Try to retrieve a scenario by its identifier.
        /// </summary>
        /// <param name="id">Scenario identifier.</param>
        /// <param name="scenario">The matching scenario definition, or null if not found.</param>
        /// <returns>True if found.</returns>
        public bool TryGetScenario(string id, out ScenarioDefinition? scenario)
        {
            return _scenarios.TryGetValue(id, out scenario);
        }

        /// <summary>
        /// Check if a scenario with the given identifier is registered.
        /// </summary>
        /// <param name="id">Scenario identifier.</param>
        /// <returns>True if registered.</returns>
        public bool Contains(string id)
        {
            return _scenarios.ContainsKey(id);
        }

        /// <summary>
        /// Register a custom scenario definition.
        /// </summary>
        /// <param name="scenario">The scenario to register.</param>
        public void Register(ScenarioDefinition scenario)
        {
            if (scenario == null) throw new ArgumentNullException(nameof(scenario));
            if (string.IsNullOrWhiteSpace(scenario.Id)) throw new ArgumentException("Scenario ID cannot be empty.", nameof(scenario));
            _scenarios[scenario.Id] = scenario;
        }

        /// <summary>
        /// Unregister a scenario by identifier.
        /// </summary>
        /// <param name="id">Scenario identifier.</param>
        /// <returns>True if a scenario was removed; false if not found.</returns>
        public bool Unregister(string id)
        {
            return _scenarios.Remove(id);
        }

        /// <summary>
        /// Get all scenarios with a specific difficulty level.
        /// </summary>
        /// <param name="difficulty">The difficulty level to filter by.</param>
        /// <returns>All scenarios matching the difficulty level.</returns>
        public IReadOnlyList<ScenarioDefinition> GetScenariosByDifficulty(Difficulty difficulty)
        {
            return _scenarios.Values
                .Where(s => s.Difficulty == difficulty)
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Get all scenarios with a specific objective type.
        /// </summary>
        /// <param name="objectiveType">The objective type to filter by.</param>
        /// <returns>All scenarios matching the objective type.</returns>
        public IReadOnlyList<ScenarioDefinition> GetScenariosByObjective(ObjectiveType objectiveType)
        {
            return _scenarios.Values
                .Where(s => s.ObjectiveType == objectiveType)
                .ToList()
                .AsReadOnly();
        }

        /// <summary>
        /// Clear all registered scenarios.
        /// </summary>
        public void Clear()
        {
            _scenarios.Clear();
        }
    }
}
