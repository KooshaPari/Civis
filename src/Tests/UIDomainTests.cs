using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for UI domain plugin including registries, models, and validation.
    /// </summary>
    public class UIDomainTests
    {
        [Fact]
        public void HudElementDefinition_DefaultConstructor_HasValidDefaults()
        {
            // Act
            HudElementDefinition element = new HudElementDefinition();

            // Assert
            element.Id.Should().Be(string.Empty);
            element.Type.Should().Be("custom");
            element.Position.Should().Be("top_left");
            element.Width.Should().Be(200);
            element.Height.Should().Be(50);
            element.VisibleIn.Should().BeEmpty();
            element.ColorOverrides.Should().BeEmpty();
            element.Opacity.Should().Be(1.0f);
        }

        [Fact]
        public void HudElementDefinition_ConstructorWithParams_InitializesCorrectly()
        {
            // Act
            HudElementDefinition element = new HudElementDefinition(
                "health-bar",
                "health_bar",
                "top_left",
                300,
                50);

            // Assert
            element.Id.Should().Be("health-bar");
            element.Type.Should().Be("health_bar");
            element.Position.Should().Be("top_left");
            element.Width.Should().Be(300);
            element.Height.Should().Be(50);
        }

        [Fact]
        public void HudElementDefinition_ConstructorWithNullId_ThrowsArgumentNullException()
        {
            // Act
            Action act = () => new HudElementDefinition(null!, "health_bar", "top_left", 300, 50);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void MenuItemDefinition_DefaultConstructor_HasValidDefaults()
        {
            // Act
            MenuItemDefinition item = new MenuItemDefinition();

            // Assert
            item.Id.Should().Be(string.Empty);
            item.Label.Should().Be(string.Empty);
            item.Action.Should().Be("custom");
            item.Target.Should().Be(string.Empty);
            item.EnabledCondition.Should().Be(string.Empty);
        }

        [Fact]
        public void MenuItemDefinition_ConstructorWithParams_InitializesCorrectly()
        {
            // Act
            MenuItemDefinition item = new MenuItemDefinition(
                "reload-packs",
                "Reload Packs",
                "reload_packs",
                "");

            // Assert
            item.Id.Should().Be("reload-packs");
            item.Label.Should().Be("Reload Packs");
            item.Action.Should().Be("reload_packs");
        }

        [Fact]
        public void MenuDefinition_DefaultConstructor_HasValidDefaults()
        {
            // Act
            MenuDefinition menu = new MenuDefinition();

            // Assert
            menu.Id.Should().Be(string.Empty);
            menu.Title.Should().Be(string.Empty);
            menu.ParentMenuId.Should().Be(string.Empty);
            menu.Items.Should().BeEmpty();
            menu.BackgroundColor.Should().Be("#1A1A1A");
            menu.FontSize.Should().Be(16);
        }

        [Fact]
        public void MenuDefinition_ConstructorWithParams_InitializesCorrectly()
        {
            // Act
            MenuDefinition menu = new MenuDefinition(
                "main-menu",
                "Main Menu",
                "");

            // Assert
            menu.Id.Should().Be("main-menu");
            menu.Title.Should().Be("Main Menu");
            menu.ParentMenuId.Should().Be(string.Empty);
        }

        [Fact]
        public void MenuDefinition_AddItems_ItemsCollectionIsPopulated()
        {
            // Arrange
            MenuDefinition menu = new MenuDefinition("main-menu", "Main Menu");
            MenuItemDefinition item1 = new MenuItemDefinition("new-game", "New Game", "custom");
            MenuItemDefinition item2 = new MenuItemDefinition("settings", "Settings", "navigate", "settings-menu");

            // Act
            menu.Items.Add(item1);
            menu.Items.Add(item2);

            // Assert
            menu.Items.Should().HaveCount(2);
            menu.Items[0].Id.Should().Be("new-game");
            menu.Items[1].Id.Should().Be("settings");
        }

        [Fact]
        public void ThemeDefinition_DefaultConstructor_HasValidDefaults()
        {
            // Act
            ThemeDefinition theme = new ThemeDefinition();

            // Assert
            theme.Id.Should().Be(string.Empty);
            theme.Name.Should().Be(string.Empty);
            theme.PrimaryColor.Should().Be("#FFFFFF");
            theme.SecondaryColor.Should().Be("#666666");
            theme.AccentColor.Should().Be("#FF6B00");
            theme.BorderRadius.Should().Be(4);
        }

        [Fact]
        public void ThemeDefinition_ConstructorWithParams_InitializesCorrectly()
        {
            // Act
            ThemeDefinition theme = new ThemeDefinition(
                "dark-theme",
                "Dark Theme",
                "#FFFFFF",
                "#999999",
                "#FF6B00");

            // Assert
            theme.Id.Should().Be("dark-theme");
            theme.Name.Should().Be("Dark Theme");
            theme.PrimaryColor.Should().Be("#FFFFFF");
        }

        [Fact]
        public void HudElementRegistry_Register_ElementCanBeRetrieved()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            HudElementDefinition element = new HudElementDefinition(
                "health-bar",
                "health_bar",
                "top_left",
                300,
                50);

            // Act
            registry.Register(element);
            HudElementDefinition retrieved = registry.GetElement("health-bar");

            // Assert
            retrieved.Should().NotBeNull();
            retrieved.Id.Should().Be("health-bar");
            retrieved.Type.Should().Be("health_bar");
        }

        [Fact]
        public void HudElementRegistry_RegisterMultiple_AllRetrievable()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            HudElementDefinition elem1 = new HudElementDefinition("health-bar", "health_bar", "top_left", 300, 50);
            HudElementDefinition elem2 = new HudElementDefinition("resource-counter", "resource_counter", "top_right", 200, 100);

            // Act
            registry.Register(elem1);
            registry.Register(elem2);

            // Assert
            registry.Count.Should().Be(2);
            registry.All.Should().HaveCount(2);
            registry.Contains("health-bar").Should().BeTrue();
            registry.Contains("resource-counter").Should().BeTrue();
        }

        [Fact]
        public void HudElementRegistry_TryGetElement_ReturnsTrueAndElement()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            HudElementDefinition element = new HudElementDefinition("health-bar", "health_bar", "top_left", 300, 50);
            registry.Register(element);

            // Act
            bool found = registry.TryGetElement("health-bar", out HudElementDefinition? retrieved);

            // Assert
            found.Should().BeTrue();
            retrieved.Should().NotBeNull();
            retrieved!.Id.Should().Be("health-bar");
        }

        [Fact]
        public void HudElementRegistry_TryGetElement_ReturnsFalseWhenNotFound()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();

            // Act
            bool found = registry.TryGetElement("nonexistent", out HudElementDefinition? retrieved);

            // Assert
            found.Should().BeFalse();
            retrieved.Should().BeNull();
        }

        [Fact]
        public void HudElementRegistry_GetElementsByType_ReturnsMatchingElements()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            registry.Register(new HudElementDefinition("health-bar", "health_bar", "top_left", 300, 50));
            registry.Register(new HudElementDefinition("player-health", "health_bar", "top_right", 250, 50));
            registry.Register(new HudElementDefinition("resource-counter", "resource_counter", "top_right", 200, 100));

            // Act
            IReadOnlyList<HudElementDefinition> healthBars = registry.GetElementsByType("health_bar");
            IReadOnlyList<HudElementDefinition> counters = registry.GetElementsByType("resource_counter");

            // Assert
            healthBars.Should().HaveCount(2);
            counters.Should().HaveCount(1);
        }

        [Fact]
        public void HudElementRegistry_GetElementsByVisibility_ReturnsVisibleElements()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            HudElementDefinition elem1 = new HudElementDefinition("gameplay-hud", "health_bar", "top_left", 300, 50);
            elem1.VisibleIn.Add("gameplay");
            elem1.VisibleIn.Add("pause");

            HudElementDefinition elem2 = new HudElementDefinition("menu-indicator", "alert_banner", "top_center", 400, 30);
            elem2.VisibleIn.Add("main_menu");

            registry.Register(elem1);
            registry.Register(elem2);

            // Act
            IReadOnlyList<HudElementDefinition> gameplayElements = registry.GetElementsByVisibility("gameplay");
            IReadOnlyList<HudElementDefinition> menuElements = registry.GetElementsByVisibility("main_menu");

            // Assert
            gameplayElements.Should().HaveCount(1);
            gameplayElements[0].Id.Should().Be("gameplay-hud");
            menuElements.Should().HaveCount(1);
            menuElements[0].Id.Should().Be("menu-indicator");
        }

        [Fact]
        public void HudElementRegistry_Unregister_ElementNoLongerRetrievable()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            HudElementDefinition element = new HudElementDefinition("health-bar", "health_bar", "top_left", 300, 50);
            registry.Register(element);

            // Act
            bool removed = registry.Unregister("health-bar");

            // Assert
            removed.Should().BeTrue();
            registry.Contains("health-bar").Should().BeFalse();
            registry.Count.Should().Be(0);
        }

        [Fact]
        public void MenuRegistry_Register_MenuCanBeRetrieved()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            MenuDefinition menu = new MenuDefinition("main-menu", "Main Menu");

            // Act
            registry.Register(menu);
            MenuDefinition retrieved = registry.GetMenu("main-menu");

            // Assert
            retrieved.Should().NotBeNull();
            retrieved.Id.Should().Be("main-menu");
            retrieved.Title.Should().Be("Main Menu");
        }

        [Fact]
        public void MenuRegistry_GetRootMenus_ReturnsMenusWithoutParent()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            MenuDefinition rootMenu = new MenuDefinition("main-menu", "Main Menu", "");
            MenuDefinition submenu = new MenuDefinition("settings-menu", "Settings", "main-menu");
            MenuDefinition otherRoot = new MenuDefinition("pause-menu", "Pause", "");

            registry.Register(rootMenu);
            registry.Register(submenu);
            registry.Register(otherRoot);

            // Act
            IReadOnlyList<MenuDefinition> roots = registry.GetRootMenus();

            // Assert
            roots.Should().HaveCount(2);
            roots.Should().Contain(m => m.Id == "main-menu");
            roots.Should().Contain(m => m.Id == "pause-menu");
        }

        [Fact]
        public void MenuRegistry_GetChildMenus_ReturnsMenusWithParent()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            MenuDefinition mainMenu = new MenuDefinition("main-menu", "Main Menu");
            MenuDefinition settingsMenu = new MenuDefinition("settings-menu", "Settings", "main-menu");
            MenuDefinition graphicsMenu = new MenuDefinition("graphics-menu", "Graphics", "settings-menu");

            registry.Register(mainMenu);
            registry.Register(settingsMenu);
            registry.Register(graphicsMenu);

            // Act
            IReadOnlyList<MenuDefinition> mainChildren = registry.GetChildMenus("main-menu");
            IReadOnlyList<MenuDefinition> settingsChildren = registry.GetChildMenus("settings-menu");

            // Assert
            mainChildren.Should().HaveCount(1);
            mainChildren[0].Id.Should().Be("settings-menu");
            settingsChildren.Should().HaveCount(1);
            settingsChildren[0].Id.Should().Be("graphics-menu");
        }

        [Fact]
        public void MenuRegistry_ValidateHierarchy_DetectsBrokenReferences()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            MenuDefinition menu = new MenuDefinition("settings-menu", "Settings", "nonexistent-parent");
            registry.Register(menu);

            // Act
            IReadOnlyList<string> errors = registry.ValidateHierarchy();

            // Assert
            errors.Should().NotBeEmpty();
            errors.Should().Contain(e => e.Contains("nonexistent-parent"));
        }

        [Fact]
        public void MenuRegistry_ValidateHierarchy_DetectsCycles()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            MenuDefinition menu1 = new MenuDefinition("menu-a", "Menu A", "menu-b");
            MenuDefinition menu2 = new MenuDefinition("menu-b", "Menu B", "menu-a");

            registry.Register(menu1);
            registry.Register(menu2);

            // Act
            IReadOnlyList<string> errors = registry.ValidateHierarchy();

            // Assert
            errors.Should().NotBeEmpty();
            errors.Should().Contain(e => e.Contains("cycle"));
        }

        [Fact]
        public void ThemeRegistry_Register_ThemeCanBeRetrieved()
        {
            // Arrange
            ThemeRegistry registry = new ThemeRegistry();
            ThemeDefinition theme = new ThemeDefinition(
                "custom-theme",
                "Custom Theme",
                "#AAAAAA",
                "#BBBBBB",
                "#CCCCCC");

            // Act
            registry.Register(theme);
            ThemeDefinition retrieved = registry.GetTheme("custom-theme");

            // Assert
            retrieved.Should().NotBeNull();
            retrieved.Id.Should().Be("custom-theme");
        }

        [Fact]
        public void ThemeRegistry_DefaultConstruction_HasDefaultTheme()
        {
            // Act
            ThemeRegistry registry = new ThemeRegistry();

            // Assert
            registry.Count.Should().BeGreaterThanOrEqualTo(2);
            registry.Contains("dark-theme").Should().BeTrue();
            registry.Contains("light-theme").Should().BeTrue();
        }

        [Fact]
        public void ThemeRegistry_SetActiveTheme_ThemeBecomesActive()
        {
            // Arrange
            ThemeRegistry registry = new ThemeRegistry();
            registry.Register(new ThemeDefinition("custom", "Custom", "#111111", "#222222", "#333333"));

            // Act
            bool success = registry.SetActiveTheme("custom");

            // Assert
            success.Should().BeTrue();
            registry.ActiveThemeId.Should().Be("custom");
            registry.ActiveTheme.Should().NotBeNull();
            registry.ActiveTheme!.Id.Should().Be("custom");
        }

        [Fact]
        public void ThemeRegistry_SetActiveTheme_InvalidThemeReturnsFalse()
        {
            // Arrange
            ThemeRegistry registry = new ThemeRegistry();

            // Act
            bool success = registry.SetActiveTheme("nonexistent");

            // Assert
            success.Should().BeFalse();
        }

        [Fact]
        public void UIPlugin_Construction_InitializesAllRegistries()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();

            // Act
            UIPlugin plugin = new UIPlugin(registries);

            // Assert
            plugin.HudElements.Should().NotBeNull();
            plugin.Menus.Should().NotBeNull();
            plugin.Themes.Should().NotBeNull();
            plugin.MenuManager.Should().NotBeNull();
            plugin.ContentLoader.Should().NotBeNull();
        }

        [Fact]
        public void UIPlugin_ValidatePack_ReportsEmptyHudElementId()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();
            UIPlugin plugin = new UIPlugin(registries);

            // Act & Assert - registry rejects empty ID on registration
            Action act = () => plugin.HudElements.Register(new HudElementDefinition { Id = "" });
            act.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void UIPlugin_ValidatePack_ReportsInvalidHudElementDimensions()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();
            UIPlugin plugin = new UIPlugin(registries);
            HudElementDefinition badElement = new HudElementDefinition("elem", "type", "pos", 0, 50);
            plugin.HudElements.Register(badElement);

            // Act
            IReadOnlyList<string> errors = plugin.ValidatePack("test-pack");

            // Assert
            errors.Should().NotBeEmpty();
            errors.Should().Contain(e => e.Contains("invalid width"));
        }

        [Fact]
        public void UIPlugin_ValidatePack_ReportsMissingMenuParent()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();
            UIPlugin plugin = new UIPlugin(registries);
            MenuDefinition menu = new MenuDefinition("sub-menu", "Submenu", "nonexistent-parent");
            plugin.Menus.Register(menu);

            // Act
            IReadOnlyList<string> errors = plugin.ValidatePack("test-pack");

            // Assert
            errors.Should().NotBeEmpty();
            errors.Should().Contain(e => e.Contains("non-existent parent"));
        }

        [Fact]
        public void UIPlugin_ValidatePack_ReportsMissingThemeColor()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();
            UIPlugin plugin = new UIPlugin(registries);
            ThemeDefinition badTheme = new ThemeDefinition { Id = "theme-1", Name = "Theme" };
            badTheme.PrimaryColor = ""; // Invalid
            plugin.Themes.Register(badTheme);

            // Act
            IReadOnlyList<string> errors = plugin.ValidatePack("test-pack");

            // Assert
            errors.Should().NotBeEmpty();
            errors.Should().Contain(e => e.Contains("primary color"));
        }

        [Fact]
        public void UIPlugin_ValidatePack_ValidPackReturnsNoErrors()
        {
            // Arrange
            RegistryManager registries = new RegistryManager();
            UIPlugin plugin = new UIPlugin(registries);

            HudElementDefinition goodElement = new HudElementDefinition("elem", "type", "top_left", 100, 100);
            plugin.HudElements.Register(goodElement);

            MenuDefinition goodMenu = new MenuDefinition("menu", "Menu", "");
            plugin.Menus.Register(goodMenu);

            ThemeDefinition goodTheme = new ThemeDefinition("theme", "Theme", "#FFFFFF", "#000000", "#FF0000");
            plugin.Themes.Register(goodTheme);

            // Act
            IReadOnlyList<string> errors = plugin.ValidatePack("test-pack");

            // Assert
            errors.Should().BeEmpty();
        }

        [Fact]
        public void HudElementRegistry_Register_NullElement_ThrowsArgumentNullException()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();

            // Act
            Action act = () => registry.Register(null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void MenuRegistry_Register_NullMenu_ThrowsArgumentNullException()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();

            // Act
            Action act = () => registry.Register(null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ThemeRegistry_Register_NullTheme_ThrowsArgumentNullException()
        {
            // Arrange
            ThemeRegistry registry = new ThemeRegistry();

            // Act
            Action act = () => registry.Register(null!);

            // Assert
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ThemeRegistry_Unregister_ActiveTheme_ReturnsFalse()
        {
            // Arrange
            ThemeRegistry registry = new ThemeRegistry();
            registry.SetActiveTheme("dark-theme");

            // Act
            bool removed = registry.Unregister("dark-theme");

            // Assert
            removed.Should().BeFalse();
            registry.Contains("dark-theme").Should().BeTrue();
        }

        [Fact]
        public void HudElementRegistry_GetElementsByType_CaseInsensitive()
        {
            // Arrange
            HudElementRegistry registry = new HudElementRegistry();
            registry.Register(new HudElementDefinition("elem", "HEALTH_BAR", "pos", 100, 100));

            // Act
            IReadOnlyList<HudElementDefinition> results = registry.GetElementsByType("health_bar");

            // Assert
            results.Should().HaveCount(1);
        }

        [Fact]
        public void MenuRegistry_GetChildMenus_EmptyParentId_ReturnsEmpty()
        {
            // Arrange
            MenuRegistry registry = new MenuRegistry();
            registry.Register(new MenuDefinition("root", "Root", ""));

            // Act
            IReadOnlyList<MenuDefinition> children = registry.GetChildMenus("");

            // Assert
            children.Should().BeEmpty();
        }
    }
}
