using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Strongly-typed representation of a DINOForge weapon definition (weapons/*.yaml).
    /// </summary>
    public class WeaponDefinition : IValidatable
    {
        /// <summary>Unique weapon identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable weapon name.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>
        /// Weapon class identifier (e.g. rifle, cannon, missile, melee).
        /// </summary>
        [YamlMember(Alias = "weapon_class")]
        public string? WeaponClass { get; set; }

        /// <summary>
        /// Damage type (e.g. kinetic, explosive, energy, melee).
        /// </summary>
        [YamlMember(Alias = "damage_type")]
        public string? DamageType { get; set; }

        /// <summary>Base damage per hit before modifiers.</summary>
        [YamlMember(Alias = "base_damage")]
        public float BaseDamage { get; set; } = 0f;

        /// <summary>Maximum attack range in world units.</summary>
        [YamlMember(Alias = "range")]
        public float Range { get; set; } = 0f;

        /// <summary>
        /// Attacks per second.
        /// </summary>
        [YamlMember(Alias = "rate_of_fire")]
        public float RateOfFire { get; set; } = 1.0f;

        /// <summary>
        /// Projectile definition ID used by this weapon.
        /// </summary>
        [YamlMember(Alias = "projectile_id")]
        public string? ProjectileId { get; set; }

        /// <summary>
        /// Area-of-effect blast radius in world units. 0 = no AoE.
        /// </summary>
        [YamlMember(Alias = "aoe_radius")]
        public float AoeRadius { get; set; } = 0f;

        /// <summary>
        /// Validates that the weapon definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            var errors = new System.Collections.Generic.List<ValidationError>();
            if (string.IsNullOrWhiteSpace(Id))
                errors.Add(new ValidationError("id", "Id is required.", "validation"));
            if (string.IsNullOrWhiteSpace(DisplayName))
                errors.Add(new ValidationError("display_name", "DisplayName is required.", "validation"));
            if (BaseDamage < 0)
                errors.Add(new ValidationError("base_damage", "BaseDamage must be non-negative.", "validation"));

            return errors.Count > 0 ? ValidationResult.Failure(errors) : ValidationResult.Success();
        }
    }
}
