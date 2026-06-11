using System.Collections.Generic;
using System.Linq;
using FluentAssertions;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using Xunit;

namespace DINOForge.Tests.SDK
{
    public sealed class DoctrineDefinitionCoverageTests
    {
        [Fact]
        public void Validate_ReturnsSuccess_WhenDefinitionIsFullyValid()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "order_legion",
                DisplayName = "Order Legion",
                Description = "Sturdy frontline doctrine.",
                FactionArchetype = "order",
                Modifiers = new Dictionary<string, float>
                {
                    ["health"] = 1.25f,
                    ["armor"] = 0f,
                },
            };

            ValidationResult result = definition.Validate();

            result.Should().NotBeNull();
            result.IsValid.Should().BeTrue();
            result.Errors.Should().NotBeNull();
            result.Errors.Should().BeEmpty();
            result.Errors.Count.Should().Be(0);
            definition.Id.Should().Be("order_legion");
            definition.DisplayName.Should().Be("Order Legion");
        }

        [Fact]
        public void Validate_AddsIdRequiredError_WhenIdIsBlank()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "   ",
                DisplayName = "Order Legion",
                Modifiers = new Dictionary<string, float>
                {
                    ["health"] = 1.0f,
                },
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors.Should().ContainSingle(error =>
                error.Path == "id" &&
                error.Message == "Id is required." &&
                error.Rule == "validation");
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Be("Id is required.");
            result.Errors[0].Rule.Should().Be("validation");
        }

        [Fact]
        public void Validate_AddsDisplayNameRequiredError_WhenDisplayNameIsBlank()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "order_legion",
                DisplayName = "",
                Modifiers = new Dictionary<string, float>
                {
                    ["health"] = 1.0f,
                },
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors.Should().ContainSingle(error =>
                error.Path == "display_name" &&
                error.Message == "DisplayName is required." &&
                error.Rule == "validation");
            result.Errors[0].Path.Should().Be("display_name");
            result.Errors[0].Message.Should().Be("DisplayName is required.");
            result.Errors[0].Rule.Should().Be("validation");
        }

        [Fact]
        public void Validate_AllowsNullModifiers_WhenRequiredFieldsAreValid()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "order_legion",
                DisplayName = "Order Legion",
                Modifiers = null!,
            };

            ValidationResult result = definition.Validate();

            result.Should().NotBeNull();
            result.IsValid.Should().BeTrue();
            result.Errors.Should().NotBeNull();
            result.Errors.Should().BeEmpty();
            result.Errors.Count.Should().Be(0);
            definition.Modifiers.Should().BeNull();
        }

        [Fact]
        public void Validate_AddsModifierValidationError_WhenASingleModifierIsNegative()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "order_legion",
                DisplayName = "Order Legion",
                Modifiers = new Dictionary<string, float>
                {
                    ["health"] = -1.25f,
                    ["armor"] = 0f,
                },
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors.Should().ContainSingle(error =>
                error.Path == "modifiers.health" &&
                error.Message == "Modifier values must be non-negative." &&
                error.Rule == "validation");
            result.Errors[0].Path.Should().Be("modifiers.health");
            result.Errors[0].Message.Should().Be("Modifier values must be non-negative.");
            result.Errors[0].Rule.Should().Be("validation");
            definition.Modifiers.Should().ContainKey("health");
            definition.Modifiers["health"].Should().Be(-1.25f);
        }

        [Fact]
        public void Validate_AddsModifierValidationErrors_ForEachNegativeModifier()
        {
            DoctrineDefinition definition = new DoctrineDefinition
            {
                Id = "asymmetric_swarm",
                DisplayName = "Asymmetric Swarm",
                Modifiers = new Dictionary<string, float>
                {
                    ["health"] = -1f,
                    ["armor"] = -0.5f,
                    ["speed"] = 1.1f,
                },
            };

            ValidationResult result = definition.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
            result.Errors.Should().Contain(error =>
                error.Path == "modifiers.health" &&
                error.Message == "Modifier values must be non-negative." &&
                error.Rule == "validation");
            result.Errors.Should().Contain(error =>
                error.Path == "modifiers.armor" &&
                error.Message == "Modifier values must be non-negative." &&
                error.Rule == "validation");
            result.Errors.Should().NotContain(error => error.Path == "modifiers.speed");
            result.Errors.Select(error => error.Path).Should().Contain(new[] { "modifiers.health", "modifiers.armor" });
            definition.Modifiers["speed"].Should().Be(1.1f);
        }
    }
}
