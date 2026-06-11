using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class SkillDefinitionCoverageTests
    {
        [Fact]
        public void Validate_ReturnsFailure_WhenIdIsMissing()
        {
            SkillDefinition skill = new SkillDefinition
            {
                Id = "",
                DisplayName = "Healing Burst",
                SkillClass = "heal",
                TargetType = "single_ally",
                Effects = new List<SkillEffect>()
            };

            ValidationResult result = skill.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().BeEmpty();
            result.Errors[0].Message.Should().Be("SkillDefinition id must not be empty");
            result.Errors[0].Rule.Should().Be("validation");
        }

        [Fact]
        public void Validate_ReturnsSuccess_WhenEffectsIsNull()
        {
            SkillDefinition skill = new SkillDefinition
            {
                Id = "heal",
                DisplayName = "Healing Burst",
                SkillClass = "heal",
                TargetType = "single_ally",
                Effects = null!
            };

            ValidationResult result = skill.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_ReturnsSuccess_WhenEffectsIsEmpty()
        {
            SkillDefinition skill = new SkillDefinition
            {
                Id = "heal",
                DisplayName = "Healing Burst",
                SkillClass = "heal",
                TargetType = "single_ally",
                Effects = new List<SkillEffect>()
            };

            ValidationResult result = skill.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_ReturnsChildValidationFailure_WhenEffectIsInvalid()
        {
            SkillDefinition skill = new SkillDefinition
            {
                Id = "rage",
                DisplayName = "Rage",
                SkillClass = "buff",
                TargetType = "single_ally",
                Effects = new List<SkillEffect>
                {
                    new SkillEffect
                    {
                        Stat = "health",
                        ModifierType = (SkillModifierType)99,
                        Value = 5f
                    }
                }
            };

            ValidationResult result = skill.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().Be("modifier_type");
            result.Errors[0].Message.Should().Be("ModifierType must be a valid SkillModifierType (Flat or Percent)");
            result.Errors[0].Rule.Should().Be("enum");
        }
    }
}
