using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class AerialPropertiesCoverageTests
    {
        [Fact]
        public void Validate_WithDefaultValues_ReturnsSuccess()
        {
            AerialProperties aerial = new();

            ValidationResult result = aerial.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithNegativeAltitudesAndSpeeds_ReturnsAllExpectedErrors()
        {
            AerialProperties aerial = new()
            {
                CruiseAltitude = -15f,
                AscendSpeed = -5f,
                DescendSpeed = -3f,
                AntiAir = true
            };

            ValidationResult result = aerial.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
            result.Errors[0].Path.Should().Be("cruise_altitude");
            result.Errors[0].Message.Should().Be("CruiseAltitude must be >= 0.");
            result.Errors[0].Rule.Should().Be("validation");
            result.Errors[1].Path.Should().Be("ascend_speed");
            result.Errors[1].Message.Should().Be("AscendSpeed must be >= 0.");
            result.Errors[1].Rule.Should().Be("validation");
            result.Errors[2].Path.Should().Be("descend_speed");
            result.Errors[2].Message.Should().Be("DescendSpeed must be >= 0.");
            result.Errors[2].Rule.Should().Be("validation");
        }
    }
}
