using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Coverage tests for <see cref="FactionDefinition"/> — exercises every branch of
    /// <see cref="FactionDefinition.Validate"/>: required Faction.Id / Faction.DisplayName
    /// and the optional hex-color validation of Visuals.PrimaryColor / AccentColor.
    /// </summary>
    public sealed class FactionDefinitionCoverageTests
    {
        private static FactionDefinition Valid() => new()
        {
            Faction = new FactionInfo { Id = "fac:rebels", DisplayName = "Rebels" }
        };

        [Fact]
        public void Validate_MinimalValid_IsValid()
        {
            Valid().Validate().IsValid.Should().BeTrue();
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_MissingFactionId_Fails(string id)
        {
            FactionDefinition d = Valid();
            d.Faction.Id = id;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "faction.id");
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_MissingFactionDisplayName_Fails(string name)
        {
            FactionDefinition d = Valid();
            d.Faction.DisplayName = name;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "faction.display_name");
        }

        [Fact]
        public void Validate_NullVisuals_SkipsColorChecks_IsValid()
        {
            FactionDefinition d = Valid();
            d.Visuals = null;

            d.Validate().IsValid.Should().BeTrue();
        }

        [Theory]
        [InlineData("#FF0000")]
        [InlineData("#00ff99")]
        [InlineData("#AbCdEf")]
        public void Validate_ValidHexColors_AreAccepted(string color)
        {
            FactionDefinition d = Valid();
            d.Visuals = new FactionVisuals { PrimaryColor = color, AccentColor = color };

            d.Validate().IsValid.Should().BeTrue();
        }

        [Theory]
        [InlineData("FF0000")]   // missing '#'
        [InlineData("#FF00")]    // too short
        [InlineData("#GG0000")]  // non-hex chars
        [InlineData("#FF000000")] // too long
        public void Validate_InvalidPrimaryColor_FailsWithPrimaryPath(string color)
        {
            FactionDefinition d = Valid();
            d.Visuals = new FactionVisuals { PrimaryColor = color };

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "visuals.primary_color");
        }

        [Fact]
        public void Validate_InvalidAccentColor_FailsWithAccentPath()
        {
            FactionDefinition d = Valid();
            d.Visuals = new FactionVisuals { AccentColor = "not-a-color" };

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "visuals.accent_color");
        }

        [Fact]
        public void Validate_NullColorFields_AreSkipped_IsValid()
        {
            FactionDefinition d = Valid();
            d.Visuals = new FactionVisuals { PrimaryColor = null, AccentColor = null };

            d.Validate().IsValid.Should().BeTrue();
        }
    }
}
