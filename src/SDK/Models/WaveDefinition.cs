using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge wave definition (waves/*.yaml).
    /// Maps to DINO's Components.WaveHolder, Components.SpawnWaveTrigger,
    /// Components.FinalWaveTrigger, and Components.EnemySpawner ECS components.
    /// </summary>
    public sealed class WaveDefinition : IValidatable
    {
        /// <summary>Unique wave identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable wave name (e.g. "Wave 1").</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional description of the wave.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Wave sequence number (1-based).
        /// </summary>
        [YamlMember(Alias = "wave_number")]
        public int WaveNumber { get; set; } = 1;

        /// <summary>
        /// Delay in seconds before this wave begins spawning.
        /// </summary>
        [YamlMember(Alias = "delay_seconds")]
        public float DelaySeconds { get; set; } = 0f;

        /// <summary>
        /// Whether this is the final wave of the scenario.
        /// Maps to Components.FinalWaveTrigger.
        /// </summary>
        [YamlMember(Alias = "is_final_wave")]
        public bool IsFinalWave { get; set; } = false;

        /// <summary>
        /// Groups of units to spawn in this wave.
        /// </summary>
        [YamlMember(Alias = "spawn_groups")]
        public List<SpawnGroup> SpawnGroups { get; set; } = new List<SpawnGroup>(); // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet

        /// <summary>
        /// Optional difficulty scaling multipliers for this wave.
        /// </summary>
        [YamlMember(Alias = "difficulty_scaling")]
        public DifficultyScaling? DifficultyScaling { get; set; }

        /// <summary>
        /// Validates that the wave definition is semantically valid.
        /// Aggregates child SpawnGroup validations (chain delegation).
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "WaveId is required.", "validation"));
            if (WaveNumber < 1)
                errors.Add(new ValidationError("wave_number", "WaveNumber must be >= 1.", "validation"));
            if (DelaySeconds < 0f)
                errors.Add(new ValidationError("delay_seconds", "DelaySeconds must be >= 0.", "validation"));

            if (DifficultyScaling != null)
            {
                if (DifficultyScaling.CountMultiplier <= 0f)
                    errors.Add(new ValidationError("difficulty_scaling.count_multiplier", "CountMultiplier must be > 0.", "validation"));
                if (DifficultyScaling.HealthMultiplier <= 0f)
                    errors.Add(new ValidationError("difficulty_scaling.health_multiplier", "HealthMultiplier must be > 0.", "validation"));
                if (DifficultyScaling.DamageMultiplier <= 0f)
                    errors.Add(new ValidationError("difficulty_scaling.damage_multiplier", "DamageMultiplier must be > 0.", "validation"));
            }

            if (SpawnGroups != null)
            {
                for (int i = 0; i < SpawnGroups.Count; i++)
                {
                    var child = SpawnGroups[i];
                    if (child == null) continue;
                    var childResult = child.Validate();
                    if (!childResult.IsValid)
                    {
                        foreach (var childErr in childResult.Errors)
                        {
                            errors.Add(new ValidationError(
                                $"spawn_groups[{i}].{childErr.Path}",
                                childErr.Message,
                                childErr.Rule));
                        }
                    }
                }
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>
    /// Difficulty scaling multipliers applied to a wave.
    /// </summary>
    public sealed class DifficultyScaling
    {
        /// <summary>
        /// Multiplier for unit count. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "count_multiplier")]
        public float CountMultiplier { get; set; } = 1.0f;

        /// <summary>
        /// Multiplier for unit health. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "health_multiplier")]
        public float HealthMultiplier { get; set; } = 1.0f;

        /// <summary>
        /// Multiplier for unit damage. Default 1.0.
        /// </summary>
        [YamlMember(Alias = "damage_multiplier")]
        public float DamageMultiplier { get; set; } = 1.0f;
    }
}
