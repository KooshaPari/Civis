using System;
using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Stub model for a DINOForge doctrine definition (doctrines/*.yaml).
    /// </summary>
    public sealed class DoctrineDefinition : IValidatable
    {
        /// <summary>Unique doctrine identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable doctrine name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional description of the doctrine's effects.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Faction archetype this doctrine is designed for
        /// (e.g. order, industrial_swarm, asymmetric, custom).
        /// </summary>
        [YamlMember(Alias = "faction_archetype")]
        public string? FactionArchetype { get; set; }

        /// <summary>
        /// Flat key/value modifiers applied by this doctrine.
        /// Keys are stat or system identifiers; values are multiplier or additive amounts.
        /// </summary>
        [YamlMember(Alias = "modifiers")]
        public Dictionary<string, float> Modifiers { get; set; } = new Dictionary<string, float>(StringComparer.Ordinal); // public-mutable-ok: YAML deserializer requires mutable Dictionary for YamlDotNet

        /// <summary>
        /// Validates that the doctrine definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "DisplayName is required.", "validation"));
            if (Modifiers != null)
            {
                foreach (var kvp in Modifiers)
                {
                    if (kvp.Value < 0)
                        errors.Add(new ValidationError($"modifiers.{kvp.Key}", "Modifier values must be non-negative.", "validation"));
                }
            }

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
