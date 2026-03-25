using System;
using System.Collections.Generic;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Defines a scripted event in a scenario: a trigger and associated effects.
    /// Events fire when their trigger condition is met.
    /// </summary>
    public class ScenarioEventDefinition
    {
        /// <summary>
        /// Unique identifier for this event.
        /// </summary>
        public string Id { get; set; }

        /// <summary>
        /// Human-readable name for the event.
        /// </summary>
        public string Name { get; set; }

        /// <summary>
        /// Event trigger type: turn, time_elapsed, condition, resource_threshold, population_threshold, etc.
        /// </summary>
        public string Trigger { get; set; }

        /// <summary>
        /// Trigger parameters dictionary. Key/value pairs depend on the Trigger type:
        /// - turn: { "turn_number": 5 }
        /// - time_elapsed: { "seconds": 60 }
        /// - condition: { "condition_expression": "population > 200" }
        /// - resource_threshold: { "resource": "gold", "amount": 100 }
        /// </summary>
        public Dictionary<string, object> TriggerParameters { get; set; }

        /// <summary>
        /// Event effect type: spawn_wave, resource_change, message, enable_faction, disable_faction, etc.
        /// </summary>
        public string Effect { get; set; }

        /// <summary>
        /// Effect parameters dictionary. Key/value pairs depend on the Effect type:
        /// - spawn_wave: { "wave_id": "wave_1", "enemy_faction": "enemy" }
        /// - resource_change: { "faction": "player", "resource": "food", "amount": 100 }
        /// - message: { "text": "The enemy approaches!", "duration_seconds": 5 }
        /// - enable_faction: { "faction_id": "ally" }
        /// </summary>
        public Dictionary<string, object> EffectParameters { get; set; }

        /// <summary>
        /// Whether this event has already fired (transient, not serialized).
        /// </summary>
        public bool HasFired { get; set; }

        /// <summary>
        /// Initializes a new scenario event definition with default values.
        /// </summary>
        public ScenarioEventDefinition()
        {
            Id = string.Empty;
            Name = string.Empty;
            Trigger = string.Empty;
            TriggerParameters = new Dictionary<string, object>();
            Effect = string.Empty;
            EffectParameters = new Dictionary<string, object>();
            HasFired = false;
        }

        /// <summary>
        /// Initializes a new scenario event definition with specified properties.
        /// </summary>
        public ScenarioEventDefinition(
            string id,
            string name,
            string trigger,
            Dictionary<string, object> triggerParameters,
            string effect,
            Dictionary<string, object> effectParameters)
        {
            Id = id ?? throw new ArgumentNullException(nameof(id));
            Name = name ?? throw new ArgumentNullException(nameof(name));
            Trigger = trigger ?? throw new ArgumentNullException(nameof(trigger));
            TriggerParameters = triggerParameters ?? new Dictionary<string, object>();
            Effect = effect ?? throw new ArgumentNullException(nameof(effect));
            EffectParameters = effectParameters ?? new Dictionary<string, object>();
            HasFired = false;
        }
    }
}
