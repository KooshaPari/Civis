using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class ProjectileDefinitionCoverageTests
    {
        [Fact]
        public void Validate_ReturnsExpectedErrors_WhenRequiredFieldsAreMissingAndSpeedIsNegative()
        {
            ProjectileDefinition definition = new ProjectileDefinition
            {
                Id = " ",
                DisplayName = "",
                Speed = -1f
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);

            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Be("Id is required.");
            result.Errors[0].Rule.Should().Be("validation");

            result.Errors[1].Path.Should().Be("display_name");
            result.Errors[1].Message.Should().Be("DisplayName is required.");
            result.Errors[1].Rule.Should().Be("validation");

            result.Errors[2].Path.Should().Be("speed");
            result.Errors[2].Message.Should().Be("Speed must be non-negative.");
            result.Errors[2].Rule.Should().Be("validation");
        }
    }
}
