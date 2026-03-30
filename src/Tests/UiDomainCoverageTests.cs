#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Targeted coverage tests for DINOForge.Domains.UI.
/// These tests focus on UIContentLoader, ThemeColorPalette, HUDElementDefinition,
/// and UIPlugin to raise coverage from 77.7% to 85%+.
/// </summary>
public class UiDomainCoverageTests
{
    // ──────────────────────── UIContentLoader tests ────────────────────────

    [Fact]
    public void UIContentLoader_WithNullHudRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new UIContentLoader(null!, new MenuRegistry(), new ThemeRegistry());

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void UIContentLoader_WithNullMenuRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new UIContentLoader(new HudElementRegistry(), null!, new ThemeRegistry());

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void UIContentLoader_WithNullThemeRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new UIContentLoader(new HudElementRegistry(), new MenuRegistry(), null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithMissingDirectory_ThrowsDirectoryNotFoundException()
    {
        var loader = new UIContentLoader(new HudElementRegistry(), new MenuRegistry(), new ThemeRegistry());
        string fakePath = Path.Combine(Path.GetTempPath(), Guid.NewGuid().ToString());

        Action action = () => loader.LoadPack(fakePath, "test-pack");

        action.Should().Throw<DirectoryNotFoundException>();
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithValidHudElements_RegistersElements()
    {
        var hudRegistry = new HudElementRegistry();
        var menuRegistry = new MenuRegistry();
        var themeRegistry = new ThemeRegistry();
        var loader = new UIContentLoader(hudRegistry, menuRegistry, themeRegistry);

        string tempDir = Path.Combine(Path.GetTempPath(), $"ui_test_{Guid.NewGuid():N}");
        string hudDir = Path.Combine(tempDir, "hud_elements");
        Directory.CreateDirectory(hudDir);

        string hudYaml = @"
elements:
  - id: test-health-bar
    type: health_bar
    position: top_left
    width: 200
    height: 50
";
        File.WriteAllText(Path.Combine(hudDir, "test.yaml"), hudYaml);

        try
        {
            loader.LoadPack(tempDir, "test-pack");

            hudRegistry.Contains("test-health-bar").Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithValidMenus_RegistersMenus()
    {
        var hudRegistry = new HudElementRegistry();
        var menuRegistry = new MenuRegistry();
        var themeRegistry = new ThemeRegistry();
        var loader = new UIContentLoader(hudRegistry, menuRegistry, themeRegistry);

        string tempDir = Path.Combine(Path.GetTempPath(), $"ui_test_{Guid.NewGuid():N}");
        string menusDir = Path.Combine(tempDir, "menus");
        Directory.CreateDirectory(menusDir);

        string menuYaml = @"
menus:
  - id: test-menu
    title: Test Menu
";
        File.WriteAllText(Path.Combine(menusDir, "test.yaml"), menuYaml);

        try
        {
            loader.LoadPack(tempDir, "test-pack");

            menuRegistry.Contains("test-menu").Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithValidThemes_RegistersThemes()
    {
        var hudRegistry = new HudElementRegistry();
        var menuRegistry = new MenuRegistry();
        var themeRegistry = new ThemeRegistry();
        var loader = new UIContentLoader(hudRegistry, menuRegistry, themeRegistry);

        string tempDir = Path.Combine(Path.GetTempPath(), $"ui_test_{Guid.NewGuid():N}");
        string themesDir = Path.Combine(tempDir, "themes");
        Directory.CreateDirectory(themesDir);

        string themeYaml = @"
themes:
  - id: test-theme
    name: Test Theme
    primary_color: '#FFFFFF'
    secondary_color: '#999999'
    accent_color: '#FF6B00'
";
        File.WriteAllText(Path.Combine(themesDir, "test.yaml"), themeYaml);

        try
        {
            loader.LoadPack(tempDir, "test-pack");

            themeRegistry.Contains("test-theme").Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithInvalidYaml_ThrowsInvalidOperationException()
    {
        var hudRegistry = new HudElementRegistry();
        var menuRegistry = new MenuRegistry();
        var themeRegistry = new ThemeRegistry();
        var loader = new UIContentLoader(hudRegistry, menuRegistry, themeRegistry);

        string tempDir = Path.Combine(Path.GetTempPath(), $"ui_test_{Guid.NewGuid():N}");
        string hudDir = Path.Combine(tempDir, "hud_elements");
        Directory.CreateDirectory(hudDir);

        string invalidYaml = "invalid: yaml: content: {{{";
        File.WriteAllText(Path.Combine(hudDir, "invalid.yaml"), invalidYaml);

        try
        {
            Action action = () => loader.LoadPack(tempDir, "test-pack");

            action.Should().Throw<InvalidOperationException>();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    [Fact]
    public void UIContentLoader_LoadPack_WithSingleElement_RegistersCorrectly()
    {
        var hudRegistry = new HudElementRegistry();
        var menuRegistry = new MenuRegistry();
        var themeRegistry = new ThemeRegistry();
        var loader = new UIContentLoader(hudRegistry, menuRegistry, themeRegistry);

        string tempDir = Path.Combine(Path.GetTempPath(), $"ui_test_{Guid.NewGuid():N}");
        string hudDir = Path.Combine(tempDir, "hud_elements");
        Directory.CreateDirectory(hudDir);

        // Array format (elements)
        string hudYaml = @"
elements:
  - id: single-element
    type: custom
    position: center
    width: 100
    height: 100
";
        File.WriteAllText(Path.Combine(hudDir, "single.yaml"), hudYaml);

        try
        {
            loader.LoadPack(tempDir, "test-pack");

            hudRegistry.Contains("single-element").Should().BeTrue();
        }
        finally
        {
            Directory.Delete(tempDir, true);
        }
    }

    // ──────────────────────── ThemeColorPalette tests ────────────────────────

    [Fact]
    public void ThemeColorPalette_GetContrastRatio_HandlesBlackOnWhite()
    {
        var white = new ThemeThemeColor { R = 255, G = 255, B = 255, A = 255 };
        var black = new ThemeThemeColor { R = 0, G = 0, B = 0, A = 255 };

        double ratio = ThemeThemeColorPalette.CalculateContrastRatio(black, white);

        ratio.Should().BeApproximately(21.0, 0.1);
    }

    [Fact]
    public void ThemeColorPalette_GetContrastRatio_HandlesWhiteOnBlack()
    {
        var white = new ThemeThemeColor { R = 255, G = 255, B = 255, A = 255 };
        var black = new ThemeThemeColor { R = 0, G = 0, B = 0, A = 255 };

        double ratio = ThemeThemeColorPalette.CalculateContrastRatio(white, black);

        ratio.Should().BeApproximately(21.0, 0.1);
    }

    [Fact]
    public void ThemeColorPalette_GetContrastRatio_HandlesSameColor()
    {
        var color = new ThemeThemeColor { R = 128, G = 128, B = 128, A = 255 };

        double ratio = ThemeThemeColorPalette.CalculateContrastRatio(color, color);

        ratio.Should().BeApproximately(1.0, 0.01);
    }

    [Fact]
    public void ThemeColorPalette_GetContrastRatio_HandlesGrayColors()
    {
        var lightGray = new ThemeThemeColor { R = 200, G = 200, B = 200, A = 255 };
        var darkGray = new ThemeThemeColor { R = 50, G = 50, B = 50, A = 255 };

        double ratio = ThemeThemeColorPalette.CalculateContrastRatio(lightGray, darkGray);

        ratio.Should().BeGreaterThan(4.5); // Should meet WCAG AA
    }

    [Fact]
    public void ThemeColorPalette_LookupColor_Primary_ReturnsCorrectly()
    {
        var palette = new ThemeThemeColorPalette
        {
            Id = "test",
            Primary = new ThemeThemeColor { R = 255, G = 0, B = 0, A = 255 }
        };

        var result = palette.GetThemeColor("primary");

        result.Should().NotBeNull();
        result!.R.Should().Be(255);
    }

    [Fact]
    public void ThemeColorPalette_LookupColor_UnknownType_ThrowsArgumentException()
    {
        var palette = new ThemeThemeColorPalette { Id = "test" };

        Action action = () => palette.GetThemeColor("unknown_type");

        action.Should().Throw<ArgumentException>().WithMessage("*Unknown color type*");
    }

    [Fact]
    public void ThemeColorPalette_LookupColor_CaseInsensitive()
    {
        var palette = new ThemeThemeColorPalette
        {
            Id = "test",
            Secondary = new ThemeThemeColor { R = 0, G = 255, B = 0, A = 255 }
        };

        var result = palette.GetThemeColor("SECONDARY");

        result.Should().NotBeNull();
        result!.G.Should().Be(255);
    }

    [Fact]
    public void ThemeColorPalette_ValidateContrast_WithValidColors_ReturnsEmpty()
    {
        var palette = new ThemeThemeColorPalette
        {
            Text = new ThemeThemeColor { R = 255, G = 255, B = 255, A = 255 },  // White text
            Background = new ThemeThemeColor { R = 0, G = 0, B = 0, A = 255 },  // Black background
            Hover = new ThemeThemeColor { R = 0, G = 0, B = 0, A = 255 }  // Black hover = high contrast with white text
        };

        var errors = palette.ValidateContrast();

        errors.Should().BeEmpty();
    }

    [Fact]
    public void ThemeColorPalette_ValidateContrast_WithInvalidColors_ReturnsErrors()
    {
        var palette = new ThemeThemeColorPalette
        {
            Text = new ThemeThemeColor { R = 128, G = 128, B = 128, A = 255 },
            Background = new ThemeThemeColor { R = 200, G = 200, B = 200, A = 255 }
        };

        var errors = palette.ValidateContrast();

        errors.Should().NotBeEmpty();
        errors.Should().Contain(e => e.Contains("WCAG AA"));
    }

    // ──────────────────────── ThemeColor tests ────────────────────────

    [Fact]
    public void ThemeColor_FromHex_Valid6Char_ReturnsCorrectColor()
    {
        var color = ThemeThemeColor.FromHex("#FF0000");

        color.R.Should().Be(255);
        color.G.Should().Be(0);
        color.B.Should().Be(0);
        color.A.Should().Be(255);
    }

    [Fact]
    public void ThemeColor_FromHex_Valid8Char_ReturnsCorrectColor()
    {
        var color = ThemeThemeColor.FromHex("#FF000080");

        color.R.Should().Be(255);
        color.G.Should().Be(0);
        color.B.Should().Be(0);
        color.A.Should().Be(128);
    }

    [Fact]
    public void ThemeColor_FromHex_EmptyString_ThrowsArgumentException()
    {
        Action action = () => ThemeThemeColor.FromHex("");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void ThemeColor_FromHex_NoHash_ThrowsArgumentException()
    {
        Action action = () => ThemeThemeColor.FromHex("FF0000");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void ThemeColor_FromHex_InvalidLength_ThrowsArgumentException()
    {
        Action action = () => ThemeThemeColor.FromHex("#FF00");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void ThemeColor_ToHex_ReturnsCorrectFormat()
    {
        var color = new ThemeThemeColor { R = 255, G = 0, B = 0, A = 255 };

        string hex = color.ToHex();

        hex.Should().StartWith("#");
        hex.Length.Should().Be(7);
    }

    [Fact]
    public void ThemeColor_Brighten_IncreasesBrightness()
    {
        var color = new ThemeThemeColor { R = 100, G = 100, B = 100, A = 255 };

        var brightened = color.Brighten(1.2);

        brightened.R.Should().BeGreaterThan(100);
        brightened.G.Should().BeGreaterThan(100);
        brightened.B.Should().BeGreaterThan(100);
    }

    [Fact]
    public void ThemeColor_Brighten_ClampsTo255()
    {
        var color = new ThemeThemeColor { R = 250, G = 250, B = 250, A = 255 };

        var brightened = color.Brighten(2.0);

        brightened.R.Should().Be(255);
        brightened.G.Should().Be(255);
        brightened.B.Should().Be(255);
    }

    [Fact]
    public void ThemeColor_Darken_DecreasesBrightness()
    {
        var color = new ThemeThemeColor { R = 200, G = 200, B = 200, A = 255 };

        var darkened = color.Darken(0.5);

        darkened.R.Should().BeLessThan(200);
        darkened.G.Should().BeLessThan(200);
        darkened.B.Should().BeLessThan(200);
    }

    // ──────────────────────── HUDElementDefinition tests ────────────────────────

    [Fact]
    public void HUDElementDefinition_DefaultConstructor_SetsDefaults()
    {
        var definition = new HudElementDefinition();

        definition.Id.Should().BeEmpty();
        definition.Type.Should().Be("custom");
        definition.Position.Should().Be("top_left");
        definition.Width.Should().Be(200);
        definition.Height.Should().Be(50);
    }

    [Fact]
    public void HUDElementDefinition_WithAllProperties_StoresCorrectly()
    {
        var definition = new HudElementDefinition
        {
            Id = "test-element",
            Type = "health_bar",
            Position = "bottom_right",
            Width = 300,
            Height = 100,
            Description = "Test element"
        };

        definition.Id.Should().Be("test-element");
        definition.Type.Should().Be("health_bar");
        definition.Position.Should().Be("bottom_right");
        definition.Width.Should().Be(300);
        definition.Height.Should().Be(100);
        definition.Description.Should().Be("Test element");
    }

    [Fact]
    public void HUDElementDefinition_WithNullId_ThrowsArgumentNullException()
    {
        Action action = () => new HudElementDefinition(null!, "type", "position", 100, 100);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void HUDElementDefinition_WithNullType_ThrowsArgumentNullException()
    {
        Action action = () => new HudElementDefinition("id", null!, "position", 100, 100);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void HUDElementDefinition_WithNullPosition_ThrowsArgumentNullException()
    {
        Action action = () => new HudElementDefinition("id", "type", null!, 100, 100);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void HUDElementDefinition_WithColorOverrides_StoresCorrectly()
    {
        var definition = new HudElementDefinition
        {
            Id = "test",
            Type = "custom",
            Position = "center"
        };
        definition.ColorOverrides["background"] = "#FF0000";
        definition.ColorOverrides["text"] = "#FFFFFF";

        definition.ColorOverrides.Should().HaveCount(2);
        definition.ColorOverrides["background"].Should().Be("#FF0000");
    }

    // ──────────────────────── UIPlugin tests ────────────────────────

    [Fact]
    public void UIPlugin_WithNullRegistries_ThrowsArgumentNullException()
    {
        Action action = () => new UIPlugin(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithEmptyPackId_ThrowsArgumentException()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        Action action = () => plugin.ValidatePack("");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithWhitespacePackId_ThrowsArgumentException()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        Action action = () => plugin.ValidatePack("   ");

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithInvalidHudElements_ReturnsErrors()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        plugin.HudElements.Register(new HudElementDefinition
        {
            Id = "invalid-hud",
            Type = "health_bar",
            Position = "top_left",
            Width = 0,  // Invalid - must be positive
            Height = 0  // Invalid - must be positive
        });

        var errors = plugin.ValidatePack("test-pack");

        errors.Should().NotBeEmpty();
        errors.Should().Contain(e => e.Contains("invalid width") || e.Contains("invalid height"));
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithInvalidMenus_ReturnsErrors()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        plugin.Menus.Register(new MenuDefinition
        {
            Id = "test-menu",
            Title = "" // Empty title
        });

        var errors = plugin.ValidatePack("test-pack");

        errors.Should().NotBeEmpty();
        errors.Should().Contain(e => e.Contains("empty title"));
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithInvalidThemes_ReturnsErrors()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        // Register a theme with invalid empty colors
        plugin.Themes.Register(new ThemeDefinition
        {
            Id = "invalid-theme",
            Name = "Test",
            PrimaryColor = "",  // Invalid - empty
            SecondaryColor = "#999999",
            AccentColor = "#FF6B00"
        });

        var errors = plugin.ValidatePack("test-pack");

        errors.Should().NotBeEmpty();
        errors.Should().Contain(e => e.Contains("empty ID") || e.Contains("empty primary color"));
    }

    [Fact]
    public void UIPlugin_ValidatePack_WithValidContent_ReturnsEmpty()
    {
        var registries = new RegistryManager();
        var plugin = new UIPlugin(registries);

        plugin.HudElements.Register(new HudElementDefinition
        {
            Id = "valid-hud",
            Type = "health_bar",
            Position = "top_left",
            Width = 200,
            Height = 50
        });

        plugin.Menus.Register(new MenuDefinition
        {
            Id = "valid-menu",
            Title = "Valid Menu"
        });

        plugin.Themes.Register(new ThemeDefinition
        {
            Id = "valid-theme",
            Name = "Valid Theme",
            PrimaryColor = "#FFFFFF",
            SecondaryColor = "#999999",
            AccentColor = "#FF6B00"
        });

        var errors = plugin.ValidatePack("test-pack");

        errors.Should().BeEmpty();
    }

    [Fact]
    public void UIPlugin_ContentTypes_ContainsExpectedTypes()
    {
        UIPlugin.ContentTypes.Should().Contain("hud_elements");
        UIPlugin.ContentTypes.Should().Contain("menus");
        UIPlugin.ContentTypes.Should().Contain("ui_panels");
        UIPlugin.ContentTypes.Should().Contain("ui_themes");
        UIPlugin.ContentTypes.Should().Contain("ui_overlays");
    }

    // ──────────────────────── HudElementRegistry tests ────────────────────────

    [Fact]
    public void HudElementRegistry_GetElement_WithNonExistentId_ThrowsKeyNotFoundException()
    {
        var registry = new HudElementRegistry();

        Action action = () => registry.GetElement("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void HudElementRegistry_TryGetElement_ReturnsCorrectValue()
    {
        var registry = new HudElementRegistry();
        var element = new HudElementDefinition
        {
            Id = "test",
            Type = "custom",
            Position = "center"
        };
        registry.Register(element);

        bool found = registry.TryGetElement("test", out var result);

        found.Should().BeTrue();
        result.Should().NotBeNull();
        result!.Id.Should().Be("test");
    }

    [Fact]
    public void HudElementRegistry_Register_WithNull_ThrowsArgumentNullException()
    {
        var registry = new HudElementRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void HudElementRegistry_Register_WithEmptyId_ThrowsArgumentException()
    {
        var registry = new HudElementRegistry();
        var element = new HudElementDefinition
        {
            Id = "  ",
            Type = "custom",
            Position = "center"
        };

        Action action = () => registry.Register(element);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void HudElementRegistry_GetElementsByType_ReturnsMatching()
    {
        var registry = new HudElementRegistry();
        registry.Register(new HudElementDefinition { Id = "health", Type = "health_bar", Position = "top_left" });
        registry.Register(new HudElementDefinition { Id = "mana", Type = "resource_counter", Position = "top_right" });
        registry.Register(new HudElementDefinition { Id = "xp", Type = "health_bar", Position = "bottom_left" });

        var results = registry.GetElementsByType("health_bar");

        results.Should().HaveCount(2);
    }

    [Fact]
    public void HudElementRegistry_GetElementsByType_WithEmptyType_ReturnsEmpty()
    {
        var registry = new HudElementRegistry();

        var results = registry.GetElementsByType("");

        results.Should().BeEmpty();
    }

    [Fact]
    public void HudElementRegistry_GetElementsByVisibility_ReturnsMatching()
    {
        var registry = new HudElementRegistry();
        var elem1 = new HudElementDefinition { Id = "elem1", Type = "custom", Position = "center" };
        elem1.VisibleIn.Add("gameplay");
        var elem2 = new HudElementDefinition { Id = "elem2", Type = "custom", Position = "center" };
        elem2.VisibleIn.Add("pause");
        registry.Register(elem1);
        registry.Register(elem2);

        var results = registry.GetElementsByVisibility("gameplay");

        results.Should().HaveCount(1);
        results[0].Id.Should().Be("elem1");
    }

    [Fact]
    public void HudElementRegistry_Unregister_ReturnsTrueForExisting()
    {
        var registry = new HudElementRegistry();
        registry.Register(new HudElementDefinition { Id = "test", Type = "custom", Position = "center" });

        bool result = registry.Unregister("test");

        result.Should().BeTrue();
        registry.Contains("test").Should().BeFalse();
    }

    [Fact]
    public void HudElementRegistry_Unregister_ReturnsFalseForNonExistent()
    {
        var registry = new HudElementRegistry();

        bool result = registry.Unregister("nonexistent");

        result.Should().BeFalse();
    }

    // ──────────────────────── MenuRegistry tests ────────────────────────

    [Fact]
    public void MenuRegistry_GetMenu_WithNonExistentId_ThrowsKeyNotFoundException()
    {
        var registry = new MenuRegistry();

        Action action = () => registry.GetMenu("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void MenuRegistry_Register_WithNull_ThrowsArgumentNullException()
    {
        var registry = new MenuRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void MenuRegistry_ValidateHierarchy_WithBrokenReference_ReturnsError()
    {
        var registry = new MenuRegistry();
        registry.Register(new MenuDefinition { Id = "child", Title = "Child", ParentMenuId = "nonexistent" });

        var errors = registry.ValidateHierarchy();

        errors.Should().Contain(e => e.Contains("non-existent parent"));
    }

    [Fact]
    public void MenuRegistry_ValidateHierarchy_WithCycle_ReturnsError()
    {
        var registry = new MenuRegistry();
        registry.Register(new MenuDefinition { Id = "menu-a", Title = "A", ParentMenuId = "menu-b" });
        registry.Register(new MenuDefinition { Id = "menu-b", Title = "B", ParentMenuId = "menu-a" });

        var errors = registry.ValidateHierarchy();

        errors.Should().Contain(e => e.Contains("cycle"));
    }

    [Fact]
    public void MenuRegistry_GetRootMenus_ReturnsCorrectMenus()
    {
        var registry = new MenuRegistry();
        registry.Register(new MenuDefinition { Id = "root", Title = "Root" });
        registry.Register(new MenuDefinition { Id = "child", Title = "Child", ParentMenuId = "root" });

        var roots = registry.GetRootMenus();

        roots.Should().HaveCount(1);
        roots[0].Id.Should().Be("root");
    }

    [Fact]
    public void MenuRegistry_GetChildMenus_ReturnsCorrectMenus()
    {
        var registry = new MenuRegistry();
        registry.Register(new MenuDefinition { Id = "root", Title = "Root" });
        registry.Register(new MenuDefinition { Id = "child1", Title = "Child 1", ParentMenuId = "root" });
        registry.Register(new MenuDefinition { Id = "child2", Title = "Child 2", ParentMenuId = "root" });

        var children = registry.GetChildMenus("root");

        children.Should().HaveCount(2);
    }

    [Fact]
    public void MenuRegistry_GetChildMenus_WithEmptyParentId_ReturnsEmpty()
    {
        var registry = new MenuRegistry();

        var children = registry.GetChildMenus("");

        children.Should().BeEmpty();
    }

    // ──────────────────────── ThemeRegistry tests ────────────────────────

    [Fact]
    public void ThemeRegistry_HasDefaultThemes()
    {
        var registry = new ThemeRegistry();

        registry.Count.Should().BeGreaterThan(0);
        registry.Contains("dark-theme").Should().BeTrue();
        registry.Contains("light-theme").Should().BeTrue();
    }

    [Fact]
    public void ThemeRegistry_GetTheme_WithNonExistentId_ThrowsKeyNotFoundException()
    {
        var registry = new ThemeRegistry();

        Action action = () => registry.GetTheme("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void ThemeRegistry_Register_WithNull_ThrowsArgumentNullException()
    {
        var registry = new ThemeRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ThemeRegistry_Register_WithEmptyId_ThrowsArgumentException()
    {
        var registry = new ThemeRegistry();

        Action action = () => registry.Register(new ThemeDefinition
        {
            Id = "  ",
            Name = "Test",
            PrimaryColor = "#FFFFFF",
            SecondaryColor = "#999999",
            AccentColor = "#FF6B00"
        });

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void ThemeRegistry_Unregister_ActiveTheme_Fails()
    {
        var registry = new ThemeRegistry();
        registry.SetActiveTheme("dark-theme");

        bool result = registry.Unregister("dark-theme");

        result.Should().BeFalse();
        registry.Contains("dark-theme").Should().BeTrue();
    }

    [Fact]
    public void ThemeRegistry_Unregister_InactiveTheme_Succeeds()
    {
        var registry = new ThemeRegistry();
        registry.Register(new ThemeDefinition
        {
            Id = "custom-theme",
            Name = "Custom",
            PrimaryColor = "#FFFFFF",
            SecondaryColor = "#999999",
            AccentColor = "#FF6B00"
        });

        bool result = registry.Unregister("custom-theme");

        result.Should().BeTrue();
        registry.Contains("custom-theme").Should().BeFalse();
    }

    [Fact]
    public void ThemeRegistry_SetActiveTheme_WithNonExistentId_ReturnsFalse()
    {
        var registry = new ThemeRegistry();

        bool result = registry.SetActiveTheme("nonexistent");

        result.Should().BeFalse();
    }

    [Fact]
    public void ThemeRegistry_SetActiveTheme_WithExistingId_ReturnsTrue()
    {
        var registry = new ThemeRegistry();

        bool result = registry.SetActiveTheme("light-theme");

        result.Should().BeTrue();
        registry.ActiveThemeId.Should().Be("light-theme");
    }

    [Fact]
    public void ThemeRegistry_ActiveTheme_ReturnsCorrectTheme()
    {
        var registry = new ThemeRegistry();
        registry.SetActiveTheme("dark-theme");

        var activeTheme = registry.ActiveTheme;

        activeTheme.Should().NotBeNull();
        activeTheme!.Id.Should().Be("dark-theme");
    }

    [Fact]
    public void ThemeRegistry_ActiveTheme_WhenNoActive_ReturnsNull()
    {
        var registry = new ThemeRegistry();
        registry.Register(new ThemeDefinition
        {
            Id = "new-theme",
            Name = "New",
            PrimaryColor = "#FFFFFF",
            SecondaryColor = "#999999",
            AccentColor = "#FF6B00"
        });

        // Set to a non-existent theme
        registry.ActiveThemeId = "nonexistent";

        registry.ActiveTheme.Should().BeNull();
    }
}
