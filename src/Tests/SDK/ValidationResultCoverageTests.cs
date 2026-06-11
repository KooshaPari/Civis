using System.Collections.Generic;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class ValidationResultCoverageTests
    {
        [Fact]
        public void Success_ReturnsValidResultWithNoErrors()
        {
            ValidationResult result = ValidationResult.Success();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Failure_WithErrorList_ReturnsInvalidResultWithSameErrors()
        {
            IReadOnlyList<ValidationError> errors = new[]
            {
                new ValidationError("packs.unit", "Missing unit id", "required"),
                new ValidationError("packs.unit.cost", "Must be non-negative", "minimum")
            };

            ValidationResult result = ValidationResult.Failure(errors);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().BeEquivalentTo(errors);
        }

        [Fact]
        public void Failure_WithMessage_CreatesSingleValidationError()
        {
            ValidationResult result = ValidationResult.Failure("Invalid manifest");

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().BeEmpty();
            result.Errors[0].Message.Should().Be("Invalid manifest");
            result.Errors[0].Rule.Should().Be("validation");
        }

        [Fact]
        public void Failure_WithPathAndMessage_CreatesSingleValidationError()
        {
            ValidationResult result = ValidationResult.Failure("packs.units[0].name", "Name is required");

            result.IsValid.Should().BeFalse();
            result.Errors.Should().ContainSingle();
            result.Errors[0].Path.Should().Be("packs.units[0].name");
            result.Errors[0].Message.Should().Be("Name is required");
            result.Errors[0].Rule.Should().Be("validation");
        }

        [Fact]
        public void ValidationError_StoresConstructorValues()
        {
            ValidationError error = new ValidationError("packs.units[0].health", "Must be positive", "minimum");

            error.Path.Should().Be("packs.units[0].health");
            error.Message.Should().Be("Must be positive");
            error.Rule.Should().Be("minimum");
        }
    }
}
