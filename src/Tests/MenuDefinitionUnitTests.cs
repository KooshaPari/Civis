using System;
using System.Collections.Generic;
using DINOForge.Domains.UI.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="MenuDefinition"/>.
    /// Tests constructor defaults, validation rules, and edge cases.
    /// </summary>
    public class MenuDefinitionUnitTests
    {
        /// <summary>
        /// Test that parameterless constructor initializes with expected defaults.
        /// </summary>
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedDefaults()
        {
            var menu = new MenuDefinition();

            menu.Id.Should().BeEmpty();
            menu.Title.Should().BeEmpty();
            menu.ParentMenuId.Should().BeEmpty();
            menu.Items.Should().NotBeNull().And.BeEmpty();
            menu.BackgroundColor.Should().Be("#1A1A1A");
            menu.FontSize.Should().Be(16);
            menu.Description.Should().BeEmpty();
        }

        /// <summary>
        /// Test that parameterized constructor with 2 parameters sets properties correctly.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithIdAndTitle_SetsCoreProperties()
        {
            var menu = new MenuDefinition("main-menu", "Main Menu");

            menu.Id.Should().Be("main-menu");
            menu.Title.Should().Be("Main Menu");
            menu.ParentMenuId.Should().BeEmpty();
        }

        /// <summary>
        /// Test that parameterized constructor with 3 parameters sets parent menu correctly.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithParentMenuId_SetsParentCorrectly()
        {
            var menu = new MenuDefinition("submenu", "Sub Menu", "main-menu");

            menu.Id.Should().Be("submenu");
            menu.Title.Should().Be("Sub Menu");
            menu.ParentMenuId.Should().Be("main-menu");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Id is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullId_ThrowsArgumentNullException()
        {
            Action act = () => new MenuDefinition(null!, "Menu Title");

            act.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Title is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullTitle_ThrowsArgumentNullException()
        {
            Action act = () => new MenuDefinition("menu-id", null!);

            act.Should().Throw<ArgumentNullException>().WithParameterName("title");
        }

        /// <summary>
        /// Test that parameterized constructor handles null ParentMenuId by converting to empty string.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullParentMenuId_ConvertsToEmptyString()
        {
            var menu = new MenuDefinition("submenu", "Sub Menu", null);

            menu.ParentMenuId.Should().BeEmpty();
        }

        /// <summary>
        /// Test that Validate() succeeds with valid MenuDefinition.
        /// </summary>
        [Fact]
        public void Validate_WithValidDefinition_ReturnsSuccess()
        {
            var menu = new MenuDefinition("settings-menu", "Settings Menu", "main-menu");

            var result = menu.Validate();

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
            var menu = new MenuDefinition { Id = id, Title = "Valid Title" };

            var result = menu.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Rule.Should().Be("non_empty");
        }

        /// <summary>
        /// Test that Validate() fails with blank Title.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankTitle_ReturnsError(string? title)
        {
            var menu = new MenuDefinition { Id = "test-menu", Title = title };

            var result = menu.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("title");
            result.Errors[0].Rule.Should().Be("non_empty");
        }

        /// <summary>
        /// Test that Validate() detects multiple errors at once.
        /// </summary>
        [Fact]
        public void Validate_WithBlankIdAndTitle_ReturnsMultipleErrors()
        {
            var menu = new MenuDefinition { Id = "", Title = "" };

            var result = menu.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(2);
            result.Errors.Should().Contain(e => e.Path == "id");
            result.Errors.Should().Contain(e => e.Path == "title");
        }

        /// <summary>
        /// Test that Items list can be populated with MenuItemDefinition instances.
        /// </summary>
        [Fact]
        public void Items_CanBePopulatedWithMenuItems()
        {
            var menu = new MenuDefinition("test-menu", "Test Menu");
            menu.Items.Add(new MenuItemDefinition { Label = "Item 1" });
            menu.Items.Add(new MenuItemDefinition { Label = "Item 2" });

            menu.Items.Should().HaveCount(2);
        }

        /// <summary>
        /// Test that FontSize can be set to different values.
        /// </summary>
        [Theory]
        [InlineData(8)]
        [InlineData(14)]
        [InlineData(24)]
        [InlineData(32)]
        public void FontSize_CanBeSetToDifferentValues(int fontSize)
        {
            var menu = new MenuDefinition("test-menu", "Test Menu") { FontSize = fontSize };

            menu.FontSize.Should().Be(fontSize);
        }

        /// <summary>
        /// Test that BackgroundColor can be customized.
        /// </summary>
        [Fact]
        public void BackgroundColor_CanBeCustomized()
        {
            var menu = new MenuDefinition("test-menu", "Test Menu")
            {
                BackgroundColor = "#2A2A2A"
            };

            menu.BackgroundColor.Should().Be("#2A2A2A");
        }

        /// <summary>
        /// Test that Description is preserved and optional.
        /// </summary>
        [Fact]
        public void Description_IsPreserved()
        {
            var menu = new MenuDefinition("test-menu", "Test Menu")
            {
                Description = "This is a test menu"
            };

            menu.Description.Should().Be("This is a test menu");
        }

        /// <summary>
        /// Test that root menu (empty ParentMenuId) validates successfully.
        /// </summary>
        [Fact]
        public void Validate_WithRootMenuEmptyParentId_ReturnsSuccess()
        {
            var menu = new MenuDefinition("main-menu", "Main Menu", "");

            var result = menu.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that submenu with parent reference validates successfully.
        /// </summary>
        [Fact]
        public void Validate_WithSubmenuHavingParentId_ReturnsSuccess()
        {
            var menu = new MenuDefinition("submenu", "Sub Menu", "main-menu");

            var result = menu.Validate();

            result.IsValid.Should().BeTrue();
        }
    }
}
