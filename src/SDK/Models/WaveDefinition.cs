using System.Collections.Generic;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge wave definition (waves/*.yaml).
    /// Maps to DINO's Components.WaveHolder, Components.SpawnWaveTrigger,
    /// Components.FinalWaveTrigger, and Components.EnemySpawner ECS components.
    /// </summary>
    public class WaveDefinition
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
        // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet
        public List<SpawnGroup> SpawnGroups { get; set; } = new List<SpawnGroup>();

        /// <summary>
        /// Optional difficulty scaling multipliers for this wave.
        /// </summary>
        [YamlMember(Alias = "difficulty_scaling")]
        public DifficultyScaling? DifficultyScaling { get; set; }
    }

    /// <summary>
    /// Difficulty scaling multipliers applied to a wave.
    /// </summary>
    public class DifficultyScaling
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
