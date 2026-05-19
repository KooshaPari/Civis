using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Stub model for a DINOForge projectile definition (projectiles/*.yaml).
    /// </summary>
    public class ProjectileDefinition : IValidatable
    {
        /// <summary>Unique projectile identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable projectile name.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>
        /// Travel speed in world units per second.
        /// </summary>
        [YamlMember(Alias = "speed")]
        public float Speed { get; set; } = 0f;

        /// <summary>Base damage dealt on impact.</summary>
        [YamlMember(Alias = "damage")]
        public float Damage { get; set; } = 0f;

        /// <summary>
        /// Area-of-effect blast radius in world units. 0 = no AoE.
        /// </summary>
        [YamlMember(Alias = "aoe_radius")]
        public float AoeRadius { get; set; } = 0f;

        /// <summary>
        /// Visual prefab asset ID for the projectile in flight.
        /// </summary>
        [YamlMember(Alias = "visual_prefab")]
        public string? VisualPrefab { get; set; }

        /// <summary>
        /// VFX asset ID played on impact.
        /// </summary>
        [YamlMember(Alias = "impact_effect")]
        public string? ImpactEffect { get; set; }

        /// <summary>
        /// Validates that the projectile definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "DisplayName is required.", "validation"));
            if (Speed < 0)
                errors.Add(new ValidationError("speed", "Speed must be non-negative.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
