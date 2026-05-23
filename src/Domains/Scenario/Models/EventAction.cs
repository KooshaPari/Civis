using System;
using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Types of actions that can be executed by scripted events.
    /// </summary>
    public enum ActionType
    {
        /// <summary>Spawn a group of units at a location.</summary>
        SpawnUnits,
        /// <summary>Grant resources to the player.</summary>
        GrantResources,
        /// <summary>Display a message to the player.</summary>
        ShowMessage,
        /// <summary>Enable a building type for construction.</summary>
        EnableBuilding,
        /// <summary>Disable a building type from construction.</summary>
        DisableBuilding,
        /// <summary>Modify the composition of future waves.</summary>
        ChangeWaveComposition,
        /// <summary>Immediately trigger a victory.</summary>
        TriggerVictory,
        /// <summary>Immediately trigger a defeat.</summary>
        TriggerDefeat
    }

    /// <summary>
    /// An action executed as part of a scripted event. Parameters are stored as
    /// key-value pairs to allow flexible, data-driven action configuration.
    /// </summary>
    public sealed class EventAction
    {
        /// <summary>
        /// The type of action to execute.
        /// </summary>
        [YamlMember(Alias = "action_type")]
        public ActionType ActionType { get; set; } = ActionType.ShowMessage;

        /// <summary>
        /// Key-value parameters for the action. Keys and expected values depend on the action type:
        /// <list type="bullet">
        /// <item><description>SpawnUnits: unit_id, count, spawn_point</description></item>
        /// <item><description>GrantResources: food, wood, stone, iron, gold</description></item>
        /// <item><description>ShowMessage: title, text</description></item>
        /// <item><description>EnableBuilding/DisableBuilding: building_id</description></item>
        /// <item><description>ChangeWaveComposition: unit_id, count_delta</description></item>
        /// <item><description>TriggerVictory/TriggerDefeat: message</description></item>
        /// </list>
        /// </summary>
        [YamlMember(Alias = "parameters")]
        public Dictionary<string, string> Parameters { get; set; } = new Dictionary<string, string>(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for scenario content
    }
}
