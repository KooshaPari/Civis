using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class SquadDefinitionCoverageTests
    {
        [Fact]
        public void Validate_ReturnsFailure_WhenIdIsMissing()
        {
            SquadDefinition squad = new SquadDefinition
            {
                Id = "",
                DisplayName = "Infantry",
                MinSize = 1,
                MaxSize = 10
            };

            ValidationResult result = squad.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Be("Squad id must not be empty");
            result.Errors[0].Rule.Should().Be("required");
        }

        [Fact]
        public void Validate_ReturnsFailure_WhenDisplayNameIsMissing()
        {
            SquadDefinition squad = new SquadDefinition
            {
                Id = "squad-1",
                DisplayName = "",
                MinSize = 1,
                MaxSize = 10
            };

            ValidationResult result = squad.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().Be("display_name");
            result.Errors[0].Message.Should().Be("Squad display_name must not be empty");
            result.Errors[0].Rule.Should().Be("required");
        }

        [Fact]
        public void Validate_ReturnsFailure_WhenMaxSizeIsLessThanMinSize()
        {
            SquadDefinition squad = new SquadDefinition
            {
                Id = "squad-1",
                DisplayName = "Infantry",
                MinSize = 20,
                MaxSize = 10
            };

            ValidationResult result = squad.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().Be("max_size");
            result.Errors[0].Message.Should().Be("MaxSize cannot be less than MinSize");
            result.Errors[0].Rule.Should().Be("constraint-violation");
        }
    }
}
