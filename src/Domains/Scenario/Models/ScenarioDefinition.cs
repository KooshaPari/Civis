using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.Domains.Scenario.Models
{
    /// <summary>
    /// Difficulty level for a scenario, affecting resource scaling and wave intensity.
    /// </summary>
    public enum Difficulty
    {
        /// <summary>Generous resources, weaker enemies.</summary>
        Easy,
        /// <summary>Standard balance.</summary>
        Normal,
        /// <summary>Reduced resources, stronger enemies.</summary>
        Hard,
        /// <summary>Minimal resources, maximum enemy intensity.</summary>
        Nightmare
    }

    /// <summary>
    /// Primary objective type for a scenario.
    /// </summary>
    public enum ObjectiveType
    {
        /// <summary>Survive all enemy waves.</summary>
        Survive,
        /// <summary>Defend a specific target structure or area.</summary>
        Defend,
        /// <summary>Destroy enemy targets.</summary>
        Attack,
        /// <summary>Reach economic or population goals.</summary>
        Prosper,
        /// <summary>Custom objective defined by scripted events.</summary>
        Custom
    }

    /// <summary>
    /// Core scenario definition model. Defines a playable scenario with objectives,
    /// difficulty settings, wave configuration, and scripted events.
    /// </summary>
    public class ScenarioDefinition : IValidatable
    {
        /// <summary>
        /// Unique scenario identifier.
        /// </summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = string.Empty;

        /// <summary>
        /// Human-readable scenario name.
        /// </summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = string.Empty;

        /// <summary>
        /// Detailed description of the scenario.
        /// </summary>
        [YamlMember(Alias = "description")]
        public string Description { get; set; } = string.Empty;

        /// <summary>
        /// Base difficulty level for this scenario.
        /// </summary>
        [YamlMember(Alias = "difficulty")]
        public Difficulty Difficulty { get; set; } = Difficulty.Normal;

        /// <summary>
        /// Primary objective type.
        /// </summary>
        [YamlMember(Alias = "objective_type")]
        public ObjectiveType ObjectiveType { get; set; } = ObjectiveType.Survive;

        /// <summary>
        /// Total number of enemy waves in the scenario. Must be positive.
        /// </summary>
        [YamlMember(Alias = "wave_count")]
        public int WaveCount { get; set; } = 1;

        /// <summary>
        /// Maximum scenario duration in seconds. Zero means unlimited.
        /// </summary>
        [YamlMember(Alias = "max_duration")]
        public int MaxDuration { get; set; } = 0;

        /// <summary>
        /// Starting resources granted to the player at scenario start.
        /// </summary>
        [YamlMember(Alias = "starting_resources")]
        public ResourceCost StartingResources { get; set; } = new ResourceCost();

        /// <summary>
        /// List of faction IDs allowed in this scenario. Empty means all factions are allowed.
        /// </summary>
        [YamlMember(Alias = "allowed_factions")]
        public List<string> AllowedFactions { get; set; } = new List<string>();

        /// <summary>
        /// Conditions that trigger a victory when met.
        /// </summary>
        [YamlMember(Alias = "victory_conditions")]
        public List<VictoryCondition> VictoryConditions { get; set; } = new List<VictoryCondition>();

        /// <summary>
        /// Conditions that trigger a defeat when met.
        /// </summary>
        [YamlMember(Alias = "defeat_conditions")]
        public List<DefeatCondition> DefeatConditions { get; set; } = new List<DefeatCondition>();

        /// <summary>
        /// Scripted events that fire during the scenario based on triggers.
        /// </summary>
        [YamlMember(Alias = "scripted_events")]
        public List<ScriptedEvent> ScriptedEvents { get; set; } = new List<ScriptedEvent>();

        /// <inheritdoc />
        /// <remarks>
        /// Task #319 — IValidatable wiring for the ScenarioContentLoader deserialize site.
        /// Rejects blank id, blank display_name, non-positive wave_count, negative max_duration,
        /// and any blank entry in allowed_factions.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "ScenarioDefinition 'id' is required.", "non_empty"));

            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "ScenarioDefinition 'display_name' is required.", "non_empty"));

            if (WaveCount <= 0)
                errors.Add(new ValidationError("wave_count", "ScenarioDefinition 'wave_count' must be > 0.", "min-value"));

            if (MaxDuration < 0)
                errors.Add(new ValidationError("max_duration", "ScenarioDefinition 'max_duration' must be >= 0.", "min-value"));

            if (AllowedFactions != null)
            {
                for (int i = 0; i < AllowedFactions.Count; i++)
                {
                    if (string.IsNullOrWhiteSpace(AllowedFactions[i]))
                    {
                        errors.Add(new ValidationError(
                            $"allowed_factions[{i}]",
                            "ScenarioDefinition 'allowed_factions' must not contain blank entries.",
                            "non_empty"));
                    }
                }
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
