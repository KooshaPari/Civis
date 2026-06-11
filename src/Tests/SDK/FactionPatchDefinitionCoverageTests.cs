using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Coverage tests for <see cref="FactionPatchDefinition"/> — exercises both branches of
    /// <see cref="FactionPatchDefinition.Validate"/>: required TargetFaction and the
    /// "at least one addition" rule across Units / Buildings / Doctrines.
    /// </summary>
    public sealed class FactionPatchDefinitionCoverageTests
    {
        private static FactionPatchDefinition Valid()
        {
            FactionPatchDefinition d = new() { TargetFaction = "fac:rebels" };
            d.Add.Units.Add("unit:trooper");
            return d;
        }

        [Fact]
        public void Validate_TargetFactionPlusOneUnit_IsValid()
        {
            Valid().Validate().IsValid.Should().BeTrue();
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_MissingTargetFaction_Fails(string target)
        {
            FactionPatchDefinition d = Valid();
            d.TargetFaction = target;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "target_faction");
        }

        [Fact]
        public void Validate_NoAdditions_FailsWithAddError()
        {
            FactionPatchDefinition d = new() { TargetFaction = "fac:rebels" };
            // Add has empty Units/Buildings/Doctrines by default.

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "add");
        }

        [Fact]
        public void Validate_OnlyBuildings_IsValid()
        {
            FactionPatchDefinition d = new() { TargetFaction = "fac:rebels" };
            d.Add.Buildings.Add("building:barracks");

            d.Validate().IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_OnlyDoctrines_IsValid()
        {
            FactionPatchDefinition d = new() { TargetFaction = "fac:rebels" };
            d.Add.Doctrines.Add("doctrine:aggressive");

            d.Validate().IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_MissingTargetAndNoAdditions_ReportsBothErrors()
        {
            FactionPatchDefinition d = new() { TargetFaction = "" };

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
        }
    }
}
