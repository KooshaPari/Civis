using YamlDotNet.Serialization;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Types of defeat conditions available in scenarios.
    /// </summary>
    public enum DefeatConditionType
    {
        /// <summary>The player's command center is destroyed.</summary>
        CommandCenterDestroyed,
        /// <summary>The player's population reaches zero.</summary>
        PopulationZero,
        /// <summary>The scenario time limit has expired without achieving victory.</summary>
        TimeExpired,
        /// <summary>A critical resource is fully depleted.</summary>
        ResourceDepleted,
        /// <summary>Custom defeat condition evaluated by scripted events.</summary>
        Custom
    }

    /// <summary>
    /// Defines a condition that, when met, triggers defeat for the player.
    /// </summary>
    public sealed class DefeatCondition
    {
        /// <summary>
        /// The type of defeat condition to evaluate.
        /// </summary>
        [YamlMember(Alias = "condition_type")]
        public DefeatConditionType ConditionType { get; set; } = DefeatConditionType.CommandCenterDestroyed;

        /// <summary>
        /// Optional numeric target value (e.g., time limit in seconds, resource threshold).
        /// Null means the condition has no numeric threshold.
        /// </summary>
        [YamlMember(Alias = "target_value")]
        public int? TargetValue { get; set; }

        /// <summary>
        /// Human-readable description of the defeat condition.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string Description { get; set; } = string.Empty;
    }
}
