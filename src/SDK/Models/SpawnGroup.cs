using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Represents a group of units to spawn together in a wave or squad.
    /// </summary>
    public sealed class SpawnGroup : IValidatable
    {
        /// <summary>
        /// The unit ID to spawn (e.g. "unit-knight").
        /// </summary>
        [YamlMember(Alias = "unit_id")]
        public string UnitId { get; set; } = "";

        /// <summary>
        /// Number of units to spawn in this group. Must be >= 1.
        /// </summary>
        [YamlMember(Alias = "count")]
        public int Count { get; set; } = 1;

        /// <summary>
        /// Delay in seconds before spawning this group within the wave.
        /// </summary>
        [YamlMember(Alias = "spawn_delay")]
        public float SpawnDelay { get; set; } = 0f;

        /// <summary>
        /// Optional spawn point location name (e.g. "north_gate", "main_entrance").
        /// </summary>
        [YamlMember(Alias = "spawn_point")]
        public string? SpawnPoint { get; set; }

        /// <summary>
        /// Validates that the spawn group is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new List<ValidationError>();

            if (string.IsNullOrWhiteSpace(UnitId))
            {
                errors.Add(new ValidationError("unit_id", "UnitId must not be empty", "required"));
            }

            if (Count < 1)
            {
                errors.Add(new ValidationError("count", "Count must be >= 1", "min-value"));
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
