using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Negative-path validation tests for Pattern #101 (task #289):
    /// stringly-typed enum discriminators converted to real enums.
    ///
    /// Each model's <c>Validate()</c> must reject out-of-range enum values that
    /// could be produced via reflection or unchecked cast — the JSON/YAML
    /// deserializer rejects unknown literals at parse time, so these tests
    /// pin the secondary defense layer.
    /// </summary>
    public class Pattern101EnumDiscriminatorTests
    {
        // ── StatOverrideEntry.Mode (StatOverrideMode) ────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void StatOverrideEntry_Validate_RejectsUnknownMode()
        {
            // Casting an arbitrary int to the enum simulates the worst-case path:
            // a deserializer (or test fixture) bypasses JsonStringEnumConverter and
            // injects a value not in StatOverrideMode { Override, Add, Multiply }.
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.hp",
                Value = 100f,
                Mode = (StatOverrideMode)42
            };

            ValidationResult result = entry.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "mode" && e.Rule == "enum");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void StatOverrideEntry_Validate_AcceptsAllKnownModes()
        {
            foreach (StatOverrideMode mode in new[] {
                StatOverrideMode.Override, StatOverrideMode.Add, StatOverrideMode.Multiply })
            {
                var entry = new StatOverrideEntry
                {
                    Target = "unit.stats.hp",
                    Value = 1f,
                    Mode = mode
                };
                entry.Validate().IsValid.Should().BeTrue($"{mode} is in the enum");
            }
        }

        // ── SkillEffect.ModifierType (SkillModifierType) ─────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void SkillEffect_Validate_RejectsUnknownModifierType()
        {
            var effect = new SkillEffect
            {
                Stat = "health",
                ModifierType = (SkillModifierType)99,
                Value = 50f
            };

            ValidationResult result = effect.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "modifier_type" && e.Rule == "enum");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void SkillEffect_Validate_AcceptsAllKnownModifierTypes()
        {
            foreach (SkillModifierType mt in new[] {
                SkillModifierType.Flat, SkillModifierType.Percent })
            {
                var effect = new SkillEffect
                {
                    Stat = "health",
                    ModifierType = mt,
                    Value = 1f
                };
                effect.Validate().IsValid.Should().BeTrue($"{mt} is in the enum");
            }
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void SkillEffect_Validate_RejectsBlankStat()
        {
            var effect = new SkillEffect
            {
                Stat = "",
                ModifierType = SkillModifierType.Flat,
                Value = 1f
            };

            ValidationResult result = effect.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "stat" && e.Rule == "required");
        }

        // ── ResourceRate.ResourceType (ResourceKind) ─────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void ResourceRate_Validate_RejectsUnknownResourceKind()
        {
            var rate = new ResourceRate
            {
                ResourceType = (ResourceKind)1234,
                BaseRate = 10f,
                Multiplier = 1.0f
            };

            ValidationResult result = rate.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "resource_type" && e.Rule == "enum");
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ResourceRate_Validate_AcceptsAllKnownResourceKinds()
        {
            foreach (ResourceKind kind in new[] {
                ResourceKind.Food, ResourceKind.Wood, ResourceKind.Stone,
                ResourceKind.Iron, ResourceKind.Gold })
            {
                var rate = new ResourceRate
                {
                    ResourceType = kind,
                    BaseRate = 10f,
                    Multiplier = 1.0f
                };
                rate.Validate().IsValid.Should().BeTrue($"{kind} is in the enum");
            }
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void ResourceRate_DefaultResourceKind_IsFood()
        {
            // Pin the contract that ResourceRate's default ResourceKind is Food
            // so that pack authors who omit resource_type get a sensible default
            // rather than the implicit (ResourceKind)0 that an int enum would imply
            // — which happens to be Food anyway, but lock it down explicitly.
            var rate = new ResourceRate { BaseRate = 1f };
            rate.ResourceType.Should().Be(ResourceKind.Food);
        }
    }
}
