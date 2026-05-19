using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using DINOForge.SDK.Validation;

namespace DINOForge.Domains.Economy.Models
{
    /// <summary>
    /// The set of resource kinds tracked by DINO's economy.
    /// </summary>
    /// <remarks>
    /// Task #310 (Pattern #101 — Stringly-Typed Enum Discriminator) — replaces the
    /// previous <c>string ResourceType</c> field on <see cref="ResourceRate"/> with
    /// a real enum. Pack YAML files keep the lowercase string literals
    /// (<c>resource_type: food|wood|stone|iron|gold</c>) — YamlDotNet maps them by
    /// enum name (case-insensitive).
    ///
    /// Resource keys used elsewhere in the economy domain (trade routes, profile
    /// multipliers, building production dictionaries) remain string-keyed so that
    /// pack authors can use lowercase YAML literals across all those surfaces.
    /// <see cref="ResourceRate.ValidResourceTypes"/> is the canonical lowercase
    /// string list for those callsites.
    /// </remarks>
    [JsonConverter(typeof(JsonStringEnumConverter))]
    public enum ResourceKind
    {
        /// <summary>Food (crops, hunting, fishing).</summary>
        Food = 0,

        /// <summary>Wood (lumber).</summary>
        Wood = 1,

        /// <summary>Stone (quarried).</summary>
        Stone = 2,

        /// <summary>Iron ore (mining).</summary>
        Iron = 3,

        /// <summary>Gold (currency).</summary>
        Gold = 4,
    }

    /// <summary>
    /// Defines a resource production or consumption rate for an economy entity.
    /// Each rate represents a single resource type with a base rate and multiplier
    /// that together determine the effective output per tick.
    /// </summary>
    public class ResourceRate : IValidatable
    {
        /// <summary>
        /// The kind of resource this rate applies to.
        /// </summary>
        /// <remarks>
        /// Task #310 — converted from <c>string</c> to <see cref="ResourceKind"/>.
        /// Defaults to <see cref="ResourceKind.Food"/> (the implicit zero value)
        /// so authors who omit <c>resource_type</c> get a deterministic kind
        /// instead of an empty string.
        /// </remarks>
        public ResourceKind ResourceType { get; set; } = ResourceKind.Food;

        /// <summary>
        /// Base production or consumption rate per tick before multipliers.
        /// Positive values indicate production; negative values indicate consumption.
        /// </summary>
        public float BaseRate { get; set; }

        /// <summary>
        /// Multiplier applied to the base rate. Defaults to 1.0 (no modification).
        /// Values greater than 1.0 increase output; values between 0 and 1.0 reduce it.
        /// </summary>
        public float Multiplier { get; set; } = 1.0f;

        /// <summary>
        /// The final computed rate after applying the multiplier to the base rate.
        /// </summary>
        public float EffectiveRate => BaseRate * Multiplier;

        /// <summary>
        /// All valid resource type identifiers in DINO's economy (lowercase form).
        /// Used by string-keyed dictionaries (production, multipliers, trade routes)
        /// that have not migrated to the <see cref="ResourceKind"/> enum.
        /// </summary>
        public static readonly string[] ValidResourceTypes = { "food", "wood", "stone", "iron", "gold" };

        /// <inheritdoc />
        /// <remarks>
        /// Task #310 — secondary defense layer on top of YAML/JSON deserialization.
        /// Catches reflection-built or unchecked-cast values that bypass parsing.
        /// </remarks>
        public ValidationResult Validate()
        {
            List<ValidationError> errors = new List<ValidationError>();

            if (!Enum.IsDefined(typeof(ResourceKind), ResourceType))
            {
                errors.Add(new ValidationError(
                    "resource_type",
                    $"ResourceRate 'resource_type' must be one of: {string.Join(", ", Enum.GetNames(typeof(ResourceKind)))}.",
                    "enum"));
            }

            return errors.Count == 0
                ? ValidationResult.Success()
                : ValidationResult.Failure(errors.AsReadOnly());
        }
    }
}
