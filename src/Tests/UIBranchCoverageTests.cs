using System;
using System.Collections.Generic;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Targeted branch coverage tests for UI domain to reach 85%+ coverage.
    /// Focuses on edge cases and conditional branches in low-coverage areas.
    /// </summary>
    public class UIBranchCoverageTests
    {
        // ── HUDInjectionSystem Edge Cases ────────────────────

        [Fact]
        public void HUDInjectionSystem_RegisterElement_NotInitialized_ThrowsException()
        {
            var system = new HUDInjectionSystem();
            var element = new HUDElementDefinition("btn1", "Button", "button", "Main");

            var action = () => system.RegisterElement(element);

            action.Should().Throw<InvalidOperationException>()
                .WithMessage("*has not been initialized*");
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_Initialized_Succeeds()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("btn1", "Button", "button", "Main");

            system.RegisterElement(element);

            system.ElementCount.Should().Be(1);
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_NullElement_ThrowsArgumentNullException()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            var action = () => system.RegisterElement(null!);

            action.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void HUDInjectionSystem_RegisterElement_Multiple_AllRegistered()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var elem1 = new HUDElementDefinition("btn1", "Button 1", "button", "Main");
            var elem2 = new HUDElementDefinition("btn2", "Button 2", "button", "Main");
            var elem3 = new HUDElementDefinition("lbl1", "Label 1", "label", "Info");

            system.RegisterElement(elem1);
            system.RegisterElement(elem2);
            system.RegisterElement(elem3);

            system.ElementCount.Should().Be(3);
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_Exists_ReturnsTrue()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("btn1", "Button", "button", "Main");
            system.RegisterElement(element);

            var result = system.UnregisterElement("btn1");

            result.Should().BeTrue();
            system.ElementCount.Should().Be(0);
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_NotExists_ReturnsFalse()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            var result = system.UnregisterElement("nonexistent");

            result.Should().BeFalse();
        }

        [Fact]
        public void HUDInjectionSystem_UnregisterElement_CaseInsensitive_ReturnsTrue()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("BTN1", "Button", "button", "Main");
            system.RegisterElement(element);

            var result = system.UnregisterElement("btn1");

            result.Should().BeTrue();
            system.ElementCount.Should().Be(0);
        }

        [Fact]
        public void HUDInjectionSystem_GetElements_Empty_ReturnsEmptyList()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            var elements = system.GetElements();

            elements.Should().BeEmpty();
        }

        [Fact]
        public void HUDInjectionSystem_GetElements_WithElements_ReturnsAll()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var elem1 = new HUDElementDefinition("btn1", "Button", "button", "Main");
            var elem2 = new HUDElementDefinition("lbl1", "Label", "label", "Info");
            system.RegisterElement(elem1);
            system.RegisterElement(elem2);

            var elements = system.GetElements();

            elements.Should().HaveCount(2);
        }

        [Fact]
        public void HUDInjectionSystem_Update_NotInitialized_ReturnsEarly()
        {
            var system = new HUDInjectionSystem();

            system.Update(); // Should not throw

            system.IsInitialized.Should().BeFalse();
        }

        [Fact]
        public void HUDInjectionSystem_Update_Initialized_Succeeds()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            system.Update(); // Should not throw

            system.IsInitialized.Should().BeTrue();
        }

        [Fact]
        public void HUDInjectionSystem_Shutdown_ClearsElementsAndState()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            var element = new HUDElementDefinition("btn1", "Button", "button", "Main");
            system.RegisterElement(element);

            system.Shutdown();

            system.IsInitialized.Should().BeFalse();
            system.ElementCount.Should().Be(0);
        }

        [Fact]
        public void HUDInjectionSystem_IsInitialized_BeforeInit_IsFalse()
        {
            var system = new HUDInjectionSystem();

            system.IsInitialized.Should().BeFalse();
        }

        [Fact]
        public void HUDInjectionSystem_IsInitialized_AfterInit_IsTrue()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();

            system.IsInitialized.Should().BeTrue();
        }

        [Fact]
        public void HUDInjectionSystem_Initialize_ClearsExistingElements()
        {
            var system = new HUDInjectionSystem();
            system.Initialize();
            system.RegisterElement(new HUDElementDefinition("btn1", "Button", "button", "Main"));
            system.ElementCount.Should().Be(1);

            system.Initialize();

            system.ElementCount.Should().Be(0);
        }

        // ── MenuRegistry Edge Cases ──────────────────────────

        [Fact]
        public void MenuRegistry_GetMenu_Exists_ReturnsMenu()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "main-menu", Title = "Main Menu" };
            registry.Register(menu);

            var result = registry.GetMenu("main-menu");

            result.Should().NotBeNull();
            result.Id.Should().Be("main-menu");
        }

        [Fact]
        public void MenuRegistry_GetMenu_NotExists_ThrowsKeyNotFoundException()
        {
            var registry = new MenuRegistry();

            var action = () => registry.GetMenu("nonexistent");

            action.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void MenuRegistry_TryGetMenu_Exists_ReturnsTrue()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "main-menu", Title = "Main Menu" };
            registry.Register(menu);

            var result = registry.TryGetMenu("main-menu", out var foundMenu);

            result.Should().BeTrue();
            foundMenu.Should().NotBeNull();
        }

        [Fact]
        public void MenuRegistry_TryGetMenu_NotExists_ReturnsFalse()
        {
            var registry = new MenuRegistry();

            var result = registry.TryGetMenu("nonexistent", out var foundMenu);

            result.Should().BeFalse();
            foundMenu.Should().BeNull();
        }

        [Fact]
        public void MenuRegistry_Register_NullMenu_ThrowsArgumentNullException()
        {
            var registry = new MenuRegistry();

            var action = () => registry.Register(null!);

            action.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void MenuRegistry_Register_EmptyId_ThrowsArgumentException()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "", Title = "Menu" };

            var action = () => registry.Register(menu);

            action.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void MenuRegistry_Register_Duplicate_Overwrites()
        {
            var registry = new MenuRegistry();
            var menu1 = new MenuDefinition { Id = "menu1", Title = "Menu 1" };
            var menu2 = new MenuDefinition { Id = "menu1", Title = "Menu 1 Updated" };

            registry.Register(menu1);
            registry.Count.Should().Be(1);
            registry.Register(menu2);

            registry.Count.Should().Be(1);
            registry.GetMenu("menu1").Title.Should().Be("Menu 1 Updated");
        }

        [Fact]
        public void MenuRegistry_Unregister_Exists_ReturnsTrue()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "menu1", Title = "Menu" };
            registry.Register(menu);

            var result = registry.Unregister("menu1");

            result.Should().BeTrue();
            registry.Count.Should().Be(0);
        }

        [Fact]
        public void MenuRegistry_Unregister_NotExists_ReturnsFalse()
        {
            var registry = new MenuRegistry();

            var result = registry.Unregister("nonexistent");

            result.Should().BeFalse();
        }

        [Fact]
        public void MenuRegistry_Contains_Exists_ReturnsTrue()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "menu1", Title = "Menu" };
            registry.Register(menu);

            registry.Contains("menu1").Should().BeTrue();
        }

        [Fact]
        public void MenuRegistry_Contains_NotExists_ReturnsFalse()
        {
            var registry = new MenuRegistry();

            registry.Contains("nonexistent").Should().BeFalse();
        }

        [Fact]
        public void MenuRegistry_GetRootMenus_NoParent_ReturnsMenu()
        {
            var registry = new MenuRegistry();
            var root = new MenuDefinition { Id = "root", Title = "Root Menu", ParentMenuId = "" };
            registry.Register(root);

            var roots = registry.GetRootMenus();

            roots.Should().HaveCount(1);
            roots[0].Id.Should().Be("root");
        }

        [Fact]
        public void MenuRegistry_GetRootMenus_WithParent_NotReturned()
        {
            var registry = new MenuRegistry();
            var root = new MenuDefinition { Id = "root", Title = "Root Menu", ParentMenuId = "" };
            var child = new MenuDefinition { Id = "child", Title = "Child Menu", ParentMenuId = "root" };
            registry.Register(root);
            registry.Register(child);

            var roots = registry.GetRootMenus();

            roots.Should().HaveCount(1);
            roots[0].Id.Should().Be("root");
        }

        [Fact]
        public void MenuRegistry_GetChildMenus_EmptyParent_ReturnsEmpty()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "child", Title = "Child", ParentMenuId = "parent" };
            registry.Register(menu);

            var children = registry.GetChildMenus("");

            children.Should().BeEmpty();
        }

        [Fact]
        public void MenuRegistry_GetChildMenus_WithChildren_ReturnsAll()
        {
            var registry = new MenuRegistry();
            var parent = new MenuDefinition { Id = "parent", Title = "Parent", ParentMenuId = "" };
            var child1 = new MenuDefinition { Id = "child1", Title = "Child 1", ParentMenuId = "parent" };
            var child2 = new MenuDefinition { Id = "child2", Title = "Child 2", ParentMenuId = "parent" };
            registry.Register(parent);
            registry.Register(child1);
            registry.Register(child2);

            var children = registry.GetChildMenus("parent");

            children.Should().HaveCount(2);
        }

        [Fact]
        public void MenuRegistry_ValidateHierarchy_Valid_ReturnsEmpty()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "menu1", Title = "Menu", ParentMenuId = "" };
            registry.Register(menu);

            var errors = registry.ValidateHierarchy();

            errors.Should().BeEmpty();
        }

        [Fact]
        public void MenuRegistry_ValidateHierarchy_BrokenReference_ReturnsError()
        {
            var registry = new MenuRegistry();
            var menu = new MenuDefinition { Id = "child", Title = "Child", ParentMenuId = "nonexistent" };
            registry.Register(menu);

            var errors = registry.ValidateHierarchy();

            errors.Should().Contain(e => e.Contains("non-existent parent"));
        }

        [Fact]
        public void MenuRegistry_ValidateHierarchy_Cycle_ReturnsError()
        {
            var registry = new MenuRegistry();
            var menu1 = new MenuDefinition { Id = "m1", Title = "Menu 1", ParentMenuId = "m2" };
            var menu2 = new MenuDefinition { Id = "m2", Title = "Menu 2", ParentMenuId = "m1" };
            registry.Register(menu1);
            registry.Register(menu2);

            var errors = registry.ValidateHierarchy();

            errors.Should().Contain(e => e.Contains("cycle"));
        }

        [Fact]
        public void MenuRegistry_All_Empty_ReturnsEmpty()
        {
            var registry = new MenuRegistry();

            registry.All.Should().BeEmpty();
        }

        [Fact]
        public void MenuRegistry_All_WithMenus_ReturnsAll()
        {
            var registry = new MenuRegistry();
            var menu1 = new MenuDefinition { Id = "m1", Title = "Menu 1" };
            var menu2 = new MenuDefinition { Id = "m2", Title = "Menu 2" };
            registry.Register(menu1);
            registry.Register(menu2);

            registry.All.Should().HaveCount(2);
        }

        // ── ThemeRegistry Edge Cases ─────────────────────────

        [Fact]
        public void ThemeRegistry_Constructor_RegistersDefaults()
        {
            var registry = new ThemeRegistry();

            registry.Count.Should().Be(2);
            registry.Contains("dark-theme").Should().BeTrue();
            registry.Contains("light-theme").Should().BeTrue();
        }

        [Fact]
        public void ThemeRegistry_ActiveTheme_Default_IsDarkTheme()
        {
            var registry = new ThemeRegistry();

            registry.ActiveThemeId.Should().Be("dark-theme");
            registry.ActiveTheme.Should().NotBeNull();
            registry.ActiveTheme!.Id.Should().Be("dark-theme");
        }

        [Fact]
        public void ThemeRegistry_GetTheme_Exists_ReturnsTheme()
        {
            var registry = new ThemeRegistry();

            var theme = registry.GetTheme("dark-theme");

            theme.Should().NotBeNull();
            theme.Id.Should().Be("dark-theme");
        }

        [Fact]
        public void ThemeRegistry_GetTheme_NotExists_ThrowsKeyNotFoundException()
        {
            var registry = new ThemeRegistry();

            var action = () => registry.GetTheme("nonexistent");

            action.Should().Throw<KeyNotFoundException>();
        }

        [Fact]
        public void ThemeRegistry_TryGetTheme_Exists_ReturnsTrue()
        {
            var registry = new ThemeRegistry();

            var result = registry.TryGetTheme("dark-theme", out var theme);

            result.Should().BeTrue();
            theme.Should().NotBeNull();
        }

        [Fact]
        public void ThemeRegistry_TryGetTheme_NotExists_ReturnsFalse()
        {
            var registry = new ThemeRegistry();

            var result = registry.TryGetTheme("nonexistent", out var theme);

            result.Should().BeFalse();
            theme.Should().BeNull();
        }

        [Fact]
        public void ThemeRegistry_Register_NullTheme_ThrowsArgumentNullException()
        {
            var registry = new ThemeRegistry();

            var action = () => registry.Register(null!);

            action.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void ThemeRegistry_Register_EmptyId_ThrowsArgumentException()
        {
            var registry = new ThemeRegistry();
            var theme = new ThemeDefinition("", "Theme", "#FFF", "#000", "#CCC");

            var action = () => registry.Register(theme);

            action.Should().Throw<ArgumentException>();
        }

        [Fact]
        public void ThemeRegistry_Register_Duplicate_Overwrites()
        {
            var registry = new ThemeRegistry();
            var theme1 = new ThemeDefinition("custom", "Custom Theme", "#FFF", "#000", "#CCC");
            var theme2 = new ThemeDefinition("custom", "Updated Custom Theme", "#EEE", "#111", "#DDD");

            registry.Register(theme1);
            var countAfterFirst = registry.Count;
            registry.Register(theme2);

            registry.Count.Should().Be(countAfterFirst);
            registry.GetTheme("custom").Name.Should().Be("Updated Custom Theme");
        }

        [Fact]
        public void ThemeRegistry_Unregister_Exists_ReturnsTrue()
        {
            var registry = new ThemeRegistry();
            var theme = new ThemeDefinition("custom", "Custom", "#FFF", "#000", "#CCC");
            registry.Register(theme);

            var result = registry.Unregister("custom");

            result.Should().BeTrue();
        }

        [Fact]
        public void ThemeRegistry_Unregister_NotExists_ReturnsFalse()
        {
            var registry = new ThemeRegistry();

            var result = registry.Unregister("nonexistent");

            result.Should().BeFalse();
        }

        [Fact]
        public void ThemeRegistry_Unregister_ActiveTheme_ReturnsFalse()
        {
            var registry = new ThemeRegistry();

            var result = registry.Unregister("dark-theme");

            result.Should().BeFalse();
            registry.Contains("dark-theme").Should().BeTrue();
        }

        [Fact]
        public void ThemeRegistry_SetActiveTheme_Exists_ReturnsTrue()
        {
            var registry = new ThemeRegistry();

            var result = registry.SetActiveTheme("light-theme");

            result.Should().BeTrue();
            registry.ActiveThemeId.Should().Be("light-theme");
        }

        [Fact]
        public void ThemeRegistry_SetActiveTheme_NotExists_ReturnsFalse()
        {
            var registry = new ThemeRegistry();
            var originalActive = registry.ActiveThemeId;

            var result = registry.SetActiveTheme("nonexistent");

            result.Should().BeFalse();
            registry.ActiveThemeId.Should().Be(originalActive);
        }

        [Fact]
        public void ThemeRegistry_ActiveThemeId_SetNull_BecomesEmpty()
        {
            var registry = new ThemeRegistry();

            registry.ActiveThemeId = null!;

            registry.ActiveThemeId.Should().Be("");
        }

        [Fact]
        public void ThemeRegistry_ActiveTheme_NoActive_ReturnsNull()
        {
            var registry = new ThemeRegistry();
            registry.ActiveThemeId = "";

            var theme = registry.ActiveTheme;

            theme.Should().BeNull();
        }

        [Fact]
        public void ThemeRegistry_Contains_Exists_ReturnsTrue()
        {
            var registry = new ThemeRegistry();

            registry.Contains("dark-theme").Should().BeTrue();
        }

        [Fact]
        public void ThemeRegistry_Contains_NotExists_ReturnsFalse()
        {
            var registry = new ThemeRegistry();

            registry.Contains("nonexistent").Should().BeFalse();
        }

        [Fact]
        public void ThemeRegistry_All_ReturnsAllThemes()
        {
            var registry = new ThemeRegistry();

            registry.All.Should().HaveCount(2);
        }

        [Fact]
        public void ThemeRegistry_Count_ReturnsCorrectCount()
        {
            var registry = new ThemeRegistry();
            var initialCount = registry.Count;

            var theme = new ThemeDefinition("custom", "Custom", "#FFF", "#000", "#CCC");
            registry.Register(theme);

            registry.Count.Should().Be(initialCount + 1);
        }
    }
}
