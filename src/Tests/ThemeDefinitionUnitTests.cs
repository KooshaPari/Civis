using System;
using System.Collections.Generic;
using DINOForge.Domains.UI.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="ThemeDefinition"/>.
    /// Tests constructor defaults, hex color validation, and edge cases.
    /// </summary>
    public class ThemeDefinitionUnitTests
    {
        /// <summary>
        /// Test that parameterless constructor initializes with expected defaults.
        /// </summary>
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedDefaults()
        {
            var theme = new ThemeDefinition();

            theme.Id.Should().BeEmpty();
            theme.Name.Should().BeEmpty();
            theme.PrimaryColor.Should().Be("#FFFFFF");
            theme.SecondaryColor.Should().Be("#666666");
            theme.AccentColor.Should().Be("#FF6B00");
            theme.FontFamily.Should().BeEmpty();
            theme.BorderRadius.Should().Be(4);
            theme.BackgroundColor.Should().Be("#0D0D0D");
            theme.TextColor.Should().Be("#FFFFFF");
            theme.Description.Should().BeEmpty();
        }

        /// <summary>
        /// Test that parameterized constructor sets properties correctly.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_SetsCoreProperties()
        {
            var theme = new ThemeDefinition(
                "dark-theme",
                "Dark Theme",
                "#1A1A1A",
                "#555555",
                "#FFA500");

            theme.Id.Should().Be("dark-theme");
            theme.Name.Should().Be("Dark Theme");
            theme.PrimaryColor.Should().Be("#1A1A1A");
            theme.SecondaryColor.Should().Be("#555555");
            theme.AccentColor.Should().Be("#FFA500");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Id is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullId_ThrowsArgumentNullException()
        {
            Action act = () => new ThemeDefinition(
                null!,
                "Test Theme",
                "#FFFFFF",
                "#666666",
                "#FF6B00");

            act.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Name is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullName_ThrowsArgumentNullException()
        {
            Action act = () => new ThemeDefinition(
                "test-id",
                null!,
                "#FFFFFF",
                "#666666",
                "#FF6B00");

            act.Should().Throw<ArgumentNullException>().WithParameterName("name");
        }

        /// <summary>
        /// Test that parameterized constructor throws when PrimaryColor is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullPrimaryColor_ThrowsArgumentNullException()
        {
            Action act = () => new ThemeDefinition(
                "test-id",
                "Test Theme",
                null!,
                "#666666",
                "#FF6B00");

            act.Should().Throw<ArgumentNullException>().WithParameterName("primaryColor");
        }

        /// <summary>
        /// Test that Validate() succeeds with valid ThemeDefinition.
        /// </summary>
        [Fact]
        public void Validate_WithValidDefinition_ReturnsSuccess()
        {
            var theme = new ThemeDefinition(
                "dark-theme",
                "Dark Theme",
                "#1A1A1A",
                "#555555",
                "#FFA500");

            var result = theme.Validate();

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
            var theme = new ThemeDefinition { Id = id! };

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        /// <summary>
        /// Test that Validate() fails with blank Name.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankName_ReturnsError(string? name)
        {
            var theme = new ThemeDefinition
            {
                Id = "test-id",
                Name = name!,
                PrimaryColor = "#FFFFFF",
                SecondaryColor = "#666666",
                AccentColor = "#FF6B00"
            };

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "name");
        }

        /// <summary>
        /// Test that Validate() fails with invalid hex color in PrimaryColor (3-digit format invalid).
        /// </summary>
        [Fact]
        public void Validate_WithInvalidPrimaryColor_ReturnsError()
        {
            var theme = new ThemeDefinition(
                "test-id",
                "Test Theme",
                "INVALID",
                "#666666",
                "#FF6B00");

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "primary_color");
        }

        /// <summary>
        /// Test that Validate() accepts valid 6-digit hex colors.
        /// </summary>
        [Theory]
        [InlineData("#FFFFFF")]
        [InlineData("#000000")]
        [InlineData("#FF6B00")]
        [InlineData("#aabbcc")]
        [InlineData("#AABBCC")]
        public void Validate_With6DigitHexColor_ReturnsSuccess(string hexColor)
        {
            var theme = new ThemeDefinition(
                "test-id",
                "Test Theme",
                hexColor,
                "#666666",
                "#FF6B00");

            var result = theme.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() accepts valid 8-digit hex colors (with alpha).
        /// </summary>
        [Theory]
        [InlineData("#FFFFFFFF")]
        [InlineData("#FF6B00FF")]
        [InlineData("#00000000")]
        [InlineData("#aabbccdd")]
        public void Validate_With8DigitHexColor_ReturnsSuccess(string hexColor)
        {
            var theme = new ThemeDefinition(
                "test-id",
                "Test Theme",
                hexColor,
                "#666666",
                "#FF6B00");

            var result = theme.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() rejects hex colors without # prefix.
        /// </summary>
        [Fact]
        public void Validate_WithHexColorMissingHashPrefix_ReturnsError()
        {
            var theme = new ThemeDefinition(
                "test-id",
                "Test Theme",
                "FFFFFF",
                "#666666",
                "#FF6B00");

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "primary_color");
        }

        /// <summary>
        /// Test that Validate() rejects hex colors with invalid characters.
        /// </summary>
        [Theory]
        [InlineData("#GGGGGG")]
        [InlineData("#12345G")]
        [InlineData("#XYZ000")]
        public void Validate_WithInvalidHexCharacters_ReturnsError(string hexColor)
        {
            var theme = new ThemeDefinition
            {
                Id = "test-id",
                Name = "Test Theme",
                PrimaryColor = hexColor,
                SecondaryColor = "#666666",
                AccentColor = "#FF6B00"
            };

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "primary_color");
        }

        /// <summary>
        /// Test that Validate() detects errors in all color fields.
        /// </summary>
        [Fact]
        public void Validate_WithMultipleInvalidColors_ReturnsAllErrors()
        {
            var theme = new ThemeDefinition
            {
                Id = "test-id",
                Name = "Test Theme",
                PrimaryColor = "INVALID1",
                SecondaryColor = "INVALID2",
                AccentColor = "INVALID3"
            };

            var result = theme.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
            result.Errors.Should().Contain(e => e.Path == "primary_color");
            result.Errors.Should().Contain(e => e.Path == "secondary_color");
            result.Errors.Should().Contain(e => e.Path == "accent_color");
        }

        /// <summary>
        /// Test that empty hex color strings in optional fields don't trigger validation error.
        /// </summary>
        [Fact]
        public void Validate_WithEmptyBackgroundColor_SkipsValidation()
        {
            var theme = new ThemeDefinition(
                "test-id",
                "Test Theme",
                "#FFFFFF",
                "#666666",
                "#FF6B00")
            {
                BackgroundColor = ""
            };

            var result = theme.Validate();

            result.IsValid.Should().BeTrue();
        }
    }
}
