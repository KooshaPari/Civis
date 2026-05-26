using System;
using System.Collections.Generic;
using DINOForge.Domains.UI.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="HudElementDefinition"/>.
    /// Tests constructor defaults, validation rules, and edge cases.
    /// </summary>
    public class HudElementDefinitionUnitTests
    {
        /// <summary>
        /// Test that parameterless constructor initializes with expected defaults.
        /// </summary>
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedDefaults()
        {
            var hud = new HudElementDefinition();

            hud.Id.Should().BeEmpty();
            hud.Type.Should().Be("custom");
            hud.Position.Should().Be("top_left");
            hud.Width.Should().Be(200);
            hud.Height.Should().Be(50);
            hud.Opacity.Should().Be(1.0f);
            hud.VisibleIn.Should().NotBeNull().And.BeEmpty();
            hud.ColorOverrides.Should().NotBeNull().And.BeEmpty();
            hud.Description.Should().BeEmpty();
        }

        /// <summary>
        /// Test that parameterized constructor sets properties correctly.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_SetsCoreProperties()
        {
            var hud = new HudElementDefinition(
                "test-hud",
                "health_bar",
                "bottom_right",
                300,
                100);

            hud.Id.Should().Be("test-hud");
            hud.Type.Should().Be("health_bar");
            hud.Position.Should().Be("bottom_right");
            hud.Width.Should().Be(300);
            hud.Height.Should().Be(100);
        }

        /// <summary>
        /// Test that parameterized constructor throws when Id is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullId_ThrowsArgumentNullException()
        {
            Action act = () => new HudElementDefinition(
                null!,
                "health_bar",
                "top_left",
                200,
                50);

            act.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Type is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullType_ThrowsArgumentNullException()
        {
            Action act = () => new HudElementDefinition(
                "test-hud",
                null!,
                "top_left",
                200,
                50);

            act.Should().Throw<ArgumentNullException>().WithParameterName("type");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Position is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullPosition_ThrowsArgumentNullException()
        {
            Action act = () => new HudElementDefinition(
                "test-hud",
                "health_bar",
                null!,
                200,
                50);

            act.Should().Throw<ArgumentNullException>().WithParameterName("position");
        }

        /// <summary>
        /// Test that Validate() succeeds with valid HudElementDefinition.
        /// </summary>
        [Fact]
        public void Validate_WithValidDefinition_ReturnsSuccess()
        {
            var hud = new HudElementDefinition(
                "player-health",
                "health_bar",
                "top_left",
                200,
                50)
            {
                Opacity = 0.8f
            };

            var result = hud.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        /// <summary>
        /// Test that Validate() fails with blank Id.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankId_ReturnsError(string? id)
        {
            var hud = new HudElementDefinition { Id = id! };

            var result = hud.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Rule.Should().Be("non_empty");
        }

        /// <summary>
        /// Test that Validate() fails with negative Opacity.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeOpacity_ReturnsError()
        {
            var hud = new HudElementDefinition(
                "test-hud",
                "health_bar",
                "top_left",
                200,
                50)
            {
                Opacity = -0.5f
            };

            var result = hud.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("opacity");
            result.Errors[0].Rule.Should().Be("range");
        }

        /// <summary>
        /// Test that Validate() fails with Opacity > 1.0.
        /// </summary>
        [Fact]
        public void Validate_WithOpcityGreaterThanOne_ReturnsError()
        {
            var hud = new HudElementDefinition(
                "test-hud",
                "health_bar",
                "top_left",
                200,
                50)
            {
                Opacity = 1.5f
            };

            var result = hud.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("opacity");
            result.Errors[0].Rule.Should().Be("range");
        }

        /// <summary>
        /// Test that Validate() succeeds with Opacity at boundary values.
        /// </summary>
        [Theory]
        [InlineData(0.0f)]
        [InlineData(0.5f)]
        [InlineData(1.0f)]
        public void Validate_WithBoundaryOpacityValues_ReturnsSuccess(float opacity)
        {
            var hud = new HudElementDefinition(
                "test-hud",
                "health_bar",
                "top_left",
                200,
                50)
            {
                Opacity = opacity
            };

            var result = hud.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() detects multiple errors at once.
        /// </summary>
        [Fact]
        public void Validate_WithMultipleErrors_ReturnsAllErrors()
        {
            var hud = new HudElementDefinition
            {
                Id = "",
                Opacity = -0.1f
            };

            var result = hud.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
            result.Errors.Should().Contain(e => e.Path == "id");
            result.Errors.Should().Contain(e => e.Path == "opacity");
        }

        /// <summary>
        /// Test that ColorOverrides can be populated with case-insensitive key handling.
        /// </summary>
        [Fact]
        public void ColorOverrides_WithCaseInsensitiveComparer_AllowsVariantCasing()
        {
            var hud = new HudElementDefinition("test", "custom", "top_left", 100, 100);
            hud.ColorOverrides["Background"] = "#FF0000";
            hud.ColorOverrides["BACKGROUND"] = "#00FF00";

            // StringComparer.OrdinalIgnoreCase should result in the last set winning
            hud.ColorOverrides.Should().ContainKey("BACKGROUND");
        }

        /// <summary>
        /// Test that VisibleIn list can be populated with multiple values.
        /// </summary>
        [Fact]
        public void VisibleIn_CanBePopulatedWithMultipleValues()
        {
            var hud = new HudElementDefinition("test", "custom", "top_left", 100, 100);
            hud.VisibleIn.Add("gameplay");
            hud.VisibleIn.Add("pause");

            hud.VisibleIn.Should().HaveCount(2);
            hud.VisibleIn.Should().Contain("gameplay").And.Contain("pause");
        }
    }
}
