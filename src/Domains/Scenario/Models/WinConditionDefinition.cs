using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Defines a winning condition for a scenario.
    /// Win conditions are evaluated each frame; the scenario ends when one is met.
    /// </summary>
    public class WinConditionDefinition
    {
        /// <summary>
        /// Unique identifier for this win condition.
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Type of win condition: eliminate_faction, reach_population, survive_duration, control_region, etc.
        /// </summary>
        public string Type { get; set; }

        /// <summary>
        /// Human-readable description of what is required to win.
        /// </summary>
        public string Description { get; set; }

        /// <summary>
        /// Parameters dictionary for condition evaluation.
        /// Key/value pairs depend on the Type:
        /// - eliminate_faction: { "target_faction": "enemy" }
        /// - reach_population: { "population_threshold": 500 }
        /// - survive_duration: { "duration_seconds": 300 }
        /// - control_region: { "region_id": "castle", "percentage": 100 }
        /// </summary>
        public Dictionary<string, object> Parameters { get; set; }

        /// <summary>
        /// Initializes a new win condition definition with default values.
        /// </summary>
        public WinConditionDefinition()
        {
            Id = string.Empty;
            Type = string.Empty;
            Description = string.Empty;
            Parameters = new Dictionary<string, object>();
        }

        /// <summary>
        /// Initializes a new win condition definition with specified properties.
        /// </summary>
        public WinConditionDefinition(
            string id,
            string type,
            string description,
            Dictionary<string, object> parameters)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Type = type ?? throw new ArgumentNullException(nameof(type));
            Description = description ?? string.Empty;
            Parameters = parameters ?? new Dictionary<string, object>();
        }
    }
}
