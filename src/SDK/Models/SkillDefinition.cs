using System.Collections.Generic;
using DINOForge.SDK.Validation;
using YamlDotNet.Serialization;

namespace DINOForge.SDK.Models
{
    /// <summary>
    /// Enum for skill modifier types.
    /// </summary>
    public enum SkillModifierType
    {
        /// <summary>Flat modifier value.</summary>
        Flat,
        /// <summary>Percent modifier value.</summary>
        Percent
    }


    /// <summary>
    /// Strongly-typed representation of a DINOForge skill definition (skills/*.yaml).
    /// Maps to DINO's Components.Skills.* ECS components (HealSkillData, RageSkillData,
    /// MeteorCastSkillData, SummonSkillData, etc.).
    /// </summary>
    public sealed class SkillDefinition : IValidatable
    {
        /// <summary>Unique skill identifier.</summary>
        [YamlMember(Alias = "id")]
        public string Id { get; set; } = "";

        /// <summary>Human-readable skill name shown in-game.</summary>
        [YamlMember(Alias = "display_name")]
        public string DisplayName { get; set; } = "";

        /// <summary>Optional description of the skill's effects.</summary>
        [YamlMember(Alias = "description")]
        public string? Description { get; set; }

        /// <summary>
        /// Skill class category. Valid values: heal, mass_heal, buff, debuff,
        /// aoe_damage, summon, resurrection, passive.
        /// </summary>
        [YamlMember(Alias = "skill_class")]
        public string SkillClass { get; set; } = "";

        /// <summary>
        /// Targeting type. Valid values: self, single_ally, single_enemy,
        /// aoe_allies, aoe_enemies, aoe_all.
        /// </summary>
        [YamlMember(Alias = "target_type")]
        public string TargetType { get; set; } = "";

        /// <summary>
        /// Cooldown in seconds between uses.
        /// </summary>
        [YamlMember(Alias = "cooldown")]
        public float Cooldown { get; set; } = 0f;

        /// <summary>
        /// Duration in seconds. 0 = instant.
        /// </summary>
        [YamlMember(Alias = "duration")]
        public float Duration { get; set; } = 0f;

        /// <summary>
        /// Effective range in world units.
        /// </summary>
        [YamlMember(Alias = "range")]
        public float Range { get; set; } = 0f;

        /// <summary>
        /// Area-of-effect radius in world units (for AOE skills).
        /// </summary>
        [YamlMember(Alias = "radius")]
        public float Radius { get; set; } = 0f;

        /// <summary>
        /// Stat effects applied by this skill.
        /// </summary>
        [YamlMember(Alias = "effects")]
        public List<SkillEffect> Effects { get; set; } = new List<SkillEffect>(); // public-mutable-ok: YAML deserialization requires mutable List<T> for YamlDotNet

        /// <summary>
        /// The vanilla DINO skill component this maps to (e.g. "Components.Skills.HealSkillData").
        /// </summary>
        [YamlMember(Alias = "vanilla_mapping")]
        public string? VanillaMapping { get; set; }

        /// <summary>
        /// Validates that the skill definition is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            if (string.IsNullOrWhiteSpace(Id))
            {
                return ValidationResult.Failure("SkillDefinition id must not be empty");
            }

            if (Effects == null)
            {
                return ValidationResult.Success();
            }

            foreach (var effect in Effects)
            {
                var effectResult = effect.Validate();
                if (!effectResult.IsValid)
                {
                    return effectResult;
                }
            }

            return ValidationResult.Success();
        }
    }

    /// <summary>
    /// A single stat modifier effect applied by a skill.
    /// </summary>
    public sealed class SkillEffect : IValidatable
    {
        /// <summary>
        /// Stat to modify (e.g. health, speed, damage, armor).
        /// </summary>
        [YamlMember(Alias = "stat")]
        public string Stat { get; set; } = "";

        /// <summary>
        /// Modifier type: Flat or Percent.
        /// </summary>
        [YamlMember(Alias = "modifier_type")]
        public SkillModifierType ModifierType { get; set; } = SkillModifierType.Flat;

        /// <summary>
        /// Modifier value. Positive = buff, negative = debuff.
        /// </summary>
        [YamlMember(Alias = "value")]
        public float Value { get; set; } = 0f;

        /// <summary>
        /// Validates that the skill effect is semantically valid.
        /// </summary>
        public ValidationResult Validate()
        {
            if (string.IsNullOrWhiteSpace(Stat))
            {
                return ValidationResult.Failure(new[] {
                    new ValidationError("stat", "Stat is required", "required")
                });
            }

            // Check ModifierType is a known enum value (Flat or Percent)
            if (!System.Enum.IsDefined(typeof(SkillModifierType), ModifierType))
            {
                return ValidationResult.Failure(new[] {
                    new ValidationError("modifier_type", "ModifierType must be a valid SkillModifierType (Flat or Percent)", "enum")
                });
            }

            return ValidationResult.Success();
        }
    }
}
