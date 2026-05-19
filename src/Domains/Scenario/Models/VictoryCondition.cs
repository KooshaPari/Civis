using YamlDotNet.Serialization;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Types of victory conditions available in scenarios.
    /// </summary>
    public enum VictoryConditionType
    {
        /// <summary>Survive a specified number of waves.</summary>
        SurviveWaves,
        /// <summary>Destroy a specific target entity or structure.</summary>
        DestroyTarget,
        /// <summary>Reach a specified population count.</summary>
        ReachPopulation,
        /// <summary>Accumulate a specified amount of a resource.</summary>
        AccumulateResource,
        /// <summary>Survive for a specified duration in seconds.</summary>
        TimeSurvival,
        /// <summary>Custom victory condition evaluated by scripted events.</summary>
        Custom
    }

    /// <summary>
    /// Defines a condition that, when met, triggers victory for the player.
    /// </summary>
    public sealed class VictoryCondition
    {
        /// <summary>
        /// The type of victory condition to evaluate.
        /// </summary>
        [YamlMember(Alias = "condition_type")]
        public VictoryConditionType ConditionType { get; set; } = VictoryConditionType.SurviveWaves;

        /// <summary>
        /// Numeric target value for the condition (e.g., wave count, population, resource amount, seconds).
        /// </summary>
        [YamlMember(Alias = "target_value")]
        public int TargetValue { get; set; } = 0;

        /// <summary>
        /// Optional target identifier for conditions that reference a specific entity, resource type, or structure.
        /// </summary>
        [YamlMember(Alias = "target_id")]
        public string? TargetId { get; set; }

        /// <summary>
        /// Human-readable description of the victory condition.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string Description { get; set; } = string.Empty;
    }
}
