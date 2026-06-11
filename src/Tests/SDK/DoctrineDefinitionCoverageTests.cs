using System.Collections.Generic;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Coverage tests for <see cref="DoctrineDefinition"/> — exercises every branch of
    /// <see cref="DoctrineDefinition.Validate"/>: required Id, required DisplayName,
    /// and non-negative Modifier values.
    /// </summary>
    public sealed class DoctrineDefinitionCoverageTests
    {
        private static DoctrineDefinition Valid() => new()
        {
            Id = "doc:aggressive",
            DisplayName = "Aggressive",
            Modifiers = new Dictionary<string, float> { ["attack"] = 1.5f }
        };

        [Fact]
        public void Validate_FullyPopulated_IsValid()
        {
            Valid().Validate().IsValid.Should().BeTrue();
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_MissingId_FailsWithIdError(string id)
        {
            DoctrineDefinition d = Valid();
            d.Id = id;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Theory]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_MissingDisplayName_FailsWithDisplayNameError(string name)
        {
            DoctrineDefinition d = Valid();
            d.DisplayName = name;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "display_name");
        }

        [Fact]
        public void Validate_NegativeModifier_FailsWithModifierPath()
        {
            DoctrineDefinition d = Valid();
            d.Modifiers["defense"] = -0.5f;

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "modifiers.defense");
        }

        [Fact]
        public void Validate_NonNegativeModifiers_AreAccepted()
        {
            DoctrineDefinition d = Valid();
            d.Modifiers["speed"] = 0f;
            d.Modifiers["range"] = 2.5f;

            d.Validate().IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_MultipleViolations_ReportsAll()
        {
            DoctrineDefinition d = new()
            {
                Id = "",
                DisplayName = "",
                Modifiers = new Dictionary<string, float> { ["x"] = -1f }
            };

            ValidationResult result = d.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
        }
    }
}
