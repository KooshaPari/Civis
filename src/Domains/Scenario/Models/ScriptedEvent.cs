using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Types of triggers that can activate a scripted event.
    /// </summary>
    public enum TriggerType
    {
        /// <summary>Triggered when a specific wave number begins.</summary>
        OnWave,
        /// <summary>Triggered when a specific elapsed time (seconds) is reached.</summary>
        OnTime,
        /// <summary>Triggered when population reaches a threshold.</summary>
        OnPopulation,
        /// <summary>Triggered when a resource amount reaches a threshold.</summary>
        OnResource,
        /// <summary>Triggered when a specific building type is constructed.</summary>
        OnBuildingBuilt,
        /// <summary>Triggered when a cumulative kill count is reached.</summary>
        OnUnitKilled
    }

    /// <summary>
    /// A scripted event that fires during a scenario when its trigger condition is met.
    /// Events execute a list of actions that modify game state.
    /// </summary>
    public sealed class ScriptedEvent
    {
        /// <summary>
        /// Unique identifier for this scripted event.
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = string.Empty;

        /// <summary>
        /// The type of trigger that activates this event.
        /// </summary>
        [YamlMember(Alias = "trigger_type")]
        public TriggerType TriggerType { get; set; } = TriggerType.OnWave;

        /// <summary>
        /// Numeric value associated with the trigger (e.g., wave number, seconds, population count).
        /// </summary>
        [YamlMember(Alias = "trigger_value")]
        public int TriggerValue { get; set; } = 0;

        /// <summary>
        /// Optional target identifier for resource or building triggers (e.g., resource name, building ID).
        /// </summary>
        [YamlMember(Alias = "trigger_target")]
        public string? TriggerTarget { get; set; }

        /// <summary>
        /// List of actions to execute when this event triggers.
        /// </summary>
        [YamlMember(Alias = "actions")]
        public List<EventAction> Actions { get; set; } = new List<EventAction>(); // public-mutable-ok: YAML deserializer requires mutable List for scenario content
    }
}
