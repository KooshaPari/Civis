using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class BuildingDefinitionCoverageTests
    {
        [Fact]
        public void Validate_returns_expected_errors_for_missing_required_fields_and_negative_health()
        {
            BuildingDefinition definition = new BuildingDefinition
            {
                Id = "",
                DisplayName = " ",
                Health = -1
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
            result.Errors.Should().SatisfyRespectively(
                first =>
                {
                    first.Path.Should().Be("id");
                    first.Message.Should().Be("Id is required.");
                    first.Rule.Should().Be("validation");
                },
                second =>
                {
                    second.Path.Should().Be("display_name");
                    second.Message.Should().Be("DisplayName is required.");
                    second.Rule.Should().Be("validation");
                },
                third =>
                {
                    third.Path.Should().Be("health");
                    third.Message.Should().Be("Health must be non-negative.");
                    third.Rule.Should().Be("validation");
                });
        }
    }
}
