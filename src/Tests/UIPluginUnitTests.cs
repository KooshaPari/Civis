using System;
using System.Collections.Generic;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    public class UIPluginUnitTests
    {
        private static RegistryManager CreateMockRegistries()
        {
            return new RegistryManager();
        }

        [Fact]
        public void Constructor_ThrowsArgumentNullException_WhenRegistriesIsNull()
        {
            Action act = () => new UIPlugin(null!);
            act.Should().Throw<ArgumentNullException>().WithParameterName("registries");
        }

        [Fact]
        public void Constructor_InitializesAllRegistries()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.HudElements.Should().NotBeNull();
            plugin.Menus.Should().NotBeNull();
            plugin.Themes.Should().NotBeNull();
        }

        [Fact]
        public void HudElements_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.HudElements.Should().NotBeNull();
        }

        [Fact]
        public void Menus_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.Menus.Should().NotBeNull();
        }

        [Fact]
        public void Themes_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.Themes.Should().NotBeNull();
        }

        [Fact]
        public void MenuManager_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.MenuManager.Should().NotBeNull();
        }

        [Fact]
        public void ContentLoader_IsNotNull_AfterConstruction()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            plugin.ContentLoader.Should().NotBeNull();
        }

        [Fact]
        public void ContentTypes_ContainsExpectedTypes()
        {
            UIPlugin.ContentTypes.Should().Contain(new[]
            {
                "ui_panels",
                "ui_themes",
                "ui_overlays",
                "hud_elements",
                "menus"
            });
        }

        [Fact]
        public void ContentTypes_IsReadOnly()
        {
            var list = UIPlugin.ContentTypes as IReadOnlyList<string>;
            list.Should().NotBeNull();
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsEmpty()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            Action act = () => plugin.ValidatePack("");
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ThrowsArgumentException_WhenPackIdIsNull()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            Action act = () => plugin.ValidatePack(null!);
            act.Should().Throw<ArgumentException>().WithParameterName("packId");
        }

        [Fact]
        public void ValidatePack_ReturnsEmptyList_WhenNoContentRegistered()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var result = plugin.ValidatePack("test-pack");

            result.Should().NotBeNull();
            result.Should().BeEmpty();
        }

        [Fact]
        public void ValidatePack_DetectsHudElementWithInvalidWidth()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var hudElement = new HudElementDefinition
            {
                Id = "hud-health",
                Type = "health-bar",
                Width = 0,
                Height = 50
            };
            plugin.HudElements.Register(hudElement);

            var result = plugin.ValidatePack("test-pack");

            result.Should().Contain(e => e.Contains("invalid width"));
        }

        [Fact]
        public void ValidatePack_DetectsHudElementWithInvalidHeight()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var hudElement = new HudElementDefinition
            {
                Id = "hud-health",
                Type = "health-bar",
                Width = 100,
                Height = -1
            };
            plugin.HudElements.Register(hudElement);

            var result = plugin.ValidatePack("test-pack");

            result.Should().Contain(e => e.Contains("invalid height"));
        }

        [Fact]
        public void ValidatePack_DetectsMenuWithEmptyTitle()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var menu = new MenuDefinition
            {
                Id = "main-menu",
                Title = ""
            };
            plugin.Menus.Register(menu);

            var result = plugin.ValidatePack("test-pack");

            result.Should().Contain(e => e.Contains("empty title"));
        }

        [Fact]
        public void ValidatePack_DetectsThemeWithEmptyName()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var theme = new ThemeDefinition
            {
                Id = "dark-theme",
                Name = "",
                PrimaryColor = "#000000",
                SecondaryColor = "#FFFFFF",
                AccentColor = "#FF0000"
            };
            plugin.Themes.Register(theme);

            var result = plugin.ValidatePack("test-pack");

            result.Should().Contain(e => e.Contains("empty name"));
        }

        [Fact]
        public void ValidatePack_DetectsThemeWithEmptySecondaryColor()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var theme = new ThemeDefinition
            {
                Id = "dark-theme",
                Name = "Dark Theme",
                PrimaryColor = "#000000",
                SecondaryColor = "",
                AccentColor = "#FF0000"
            };
            plugin.Themes.Register(theme);

            var result = plugin.ValidatePack("test-pack");

            result.Should().Contain(e => e.Contains("secondary color"));
        }

        [Fact]
        public void ValidatePack_PassesWithValidContent()
        {
            var registries = CreateMockRegistries();
            var plugin = new UIPlugin(registries);

            var hudElement = new HudElementDefinition
            {
                Id = "hud-health",
                Type = "health-bar",
                Width = 100,
                Height = 50
            };
            plugin.HudElements.Register(hudElement);

            var menu = new MenuDefinition
            {
                Id = "main-menu",
                Title = "Main Menu"
            };
            plugin.Menus.Register(menu);

            var theme = new ThemeDefinition
            {
                Id = "dark-theme",
                Name = "Dark Theme",
                PrimaryColor = "#000000",
                SecondaryColor = "#FFFFFF",
                AccentColor = "#FF0000"
            };
            plugin.Themes.Register(theme);

            var result = plugin.ValidatePack("test-pack");

            result.Should().BeEmpty();
        }
    }
}
