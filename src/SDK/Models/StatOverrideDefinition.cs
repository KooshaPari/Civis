#nullable enable

using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Enum for stat override modes.
    /// </summary>
    public enum StatOverrideMode
    {
        /// <summary>Override mode: replace the value.</summary>
        Override,
        /// <summary>Add mode: add to the value.</summary>
        Add,
        /// <summary>Multiply mode: multiply the value.</summary>
        Multiply
    }


    /// <summary>
    /// Represents a collection of stat overrides loaded from a pack's stats YAML file.
    /// Each override targets a specific model path and applies a value modification.
    /// </summary>
    public sealed class StatOverrideDefinition : IValidatable
    {
        /// <summary>
        /// The list of stat override entries to apply.
        /// </summary>
        [YamlMember(Alias = "overrides")]
        public List<StatOverrideEntry> Overrides { get; set; } = new List<StatOverrideEntry>(); // public-mutable-ok: YAML deserializer requires mutable List<T> for YamlDotNet

        /// <summary>
        /// Validates that the stat override definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (Overrides == null || Overrides.Count == 0)
            {
                errors.Add(new ValidationError("overrides", "At least one override entry is required.", "validation"));
            }
            else
            {
                for (int i = 0; i < Overrides.Count; i++)
                {
                    var entry = Overrides[i];
                    if (string.IsNullOrWhiteSpace(entry.Target))
                        errors.Add(new ValidationError($"overrides[{i}].target", "Target must not be empty.", "validation"));
                    if (!System.Enum.IsDefined(typeof(StatOverrideMode), entry.Mode))
                        errors.Add(new ValidationError($"overrides[{i}].mode", "Mode must be a valid StatOverrideMode (Override, Add, or Multiply).", "validation"));
                }
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }

    /// <summary>
    /// A single stat override entry that targets a specific model path
    /// and applies a value using the specified mode.
    /// </summary>
    public sealed class StatOverrideEntry : IValidatable
    {
        /// <summary>
        /// SDK model path to target (e.g. "unit.stats.hp").
        /// </summary>
        [YamlMember(Alias = "target")]
        public string Target { get; set; } = "";

        /// <summary>
        /// The numeric value to apply.
        /// </summary>
        [YamlMember(Alias = "value")]
        public float Value { get; set; }

        /// <summary>
        /// How the value is applied: Override replaces, Add adds, Multiply multiplies.
        /// Defaults to Override.
        /// </summary>
        [YamlMember(Alias = "mode")]
        public StatOverrideMode Mode { get; set; } = StatOverrideMode.Override;

        /// <summary>
        /// Optional ECS component filter (e.g. "Components.MeleeUnit").
        /// When set, the override only applies to entities matching this filter.
        /// </summary>
        [YamlMember(Alias = "filter")]
        public string? Filter { get; set; }

        /// <summary>
        /// Validates that the stat override entry is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            if (string.IsNullOrWhiteSpace(Target))
            {
                return ValidationResult.Failure("StatOverrideEntry target must not be empty");
            }

            // Check Mode is a known enum value (Override, Add, or Multiply)
            if (!System.Enum.IsDefined(typeof(StatOverrideMode), Mode))
            {
                return ValidationResult.Failure(new[] {
                    new ValidationError("mode", "Mode must be a valid StatOverrideMode (Override, Add, or Multiply)", "enum")
                });
            }

            return ValidationResult.Success();
        }
    }
}
