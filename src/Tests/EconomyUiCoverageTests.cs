#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Registries;
using DINOForge.Domains.UI;
using DINOForge.Domains.UI.Models;
using DINOForge.Domains.UI.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

public class EconomyUiCoverageTests
{
    [Fact]
    public void ResourceDefinition_DefaultsAndCtorCoverProperties()
    {
        ResourceDefinition defaults = new();
        defaults.Id.Should().BeEmpty();
        defaults.Name.Should().BeEmpty();
        defaults.Description.Should().BeEmpty();
        defaults.ProductionRate.Should().Be(1.0f);
        defaults.StorageCapacity.Should().Be(1000.0f);
        defaults.DecayRate.Should().Be(0.0f);
        defaults.IsTradeableDefault.Should().BeTrue();

        ResourceDefinition custom = new("iron", "Iron", "Metal", 2.5f, 75f, 0.1f, false);
        custom.Id.Should().Be("iron");
        custom.Name.Should().Be("Iron");
        custom.Description.Should().Be("Metal");
        custom.ProductionRate.Should().Be(2.5f);
        custom.StorageCapacity.Should().Be(75f);
        custom.DecayRate.Should().Be(0.1f);
        custom.IsTradeableDefault.Should().BeFalse();
    }

    [Fact]
    public void MarketDefinition_MethodsAndDefaultsWork()
    {
        MarketDefinition market = new();
        market.Id.Should().BeEmpty();
        market.DisplayName.Should().BeEmpty();
        market.Description.Should().BeEmpty();
        market.ThroughputModifier.Should().Be(1.0f);
        market.GetPriceModifier("wood").Should().Be(1.0f);
        market.CanTrade("wood").Should().BeFalse();

        market.ResourcesTradeable.Add("wood");
        market.PriceModifiers["wood"] = 0.8f;
        market.GetPriceModifier("wood").Should().Be(0.8f);
        market.CanTrade("wood").Should().BeTrue();

        MarketDefinition custom = new(
            "market-1",
            "Central Market",
            "Trading hub",
            new List<string> { "gold" },
            new Dictionary<string, float> { ["gold"] = 1.25f },
            1.5f);
        custom.Id.Should().Be("market-1");
        custom.DisplayName.Should().Be("Central Market");
        custom.GetPriceModifier("gold").Should().Be(1.25f);
        custom.CanTrade("gold").Should().BeTrue();
    }

    [Fact]
    public void TradeRouteDefinition_DefaultsAndCtorCoverProperties()
    {
        TradeRouteDefinition defaults = new();
        defaults.Id.Should().BeEmpty();
        defaults.DisplayName.Should().BeEmpty();
        defaults.SourceResource.Should().BeEmpty();
        defaults.TargetResource.Should().BeEmpty();
        defaults.ExchangeRate.Should().Be(1.0f);
        defaults.CooldownTicks.Should().Be(60);
        defaults.MaxPerTransaction.Should().Be(1000.0f);
        defaults.Enabled.Should().BeTrue();

        TradeRouteDefinition custom = new(
            "route-1",
            "Wood to Gold",
            "wood",
            "gold",
            10f,
            120,
            500f,
            false);
        custom.Id.Should().Be("route-1");
        custom.DisplayName.Should().Be("Wood to Gold");
        custom.SourceResource.Should().Be("wood");
        custom.TargetResource.Should().Be("gold");
        custom.ExchangeRate.Should().Be(10f);
        custom.CooldownTicks.Should().Be(120);
        custom.MaxPerTransaction.Should().Be(500f);
        custom.Enabled.Should().BeFalse();
    }

    [Fact]
    public void ResourceRegistry_CoversRegistrationAndLookup()
    {
        ResourceRegistry registry = new();
        registry.Count.Should().Be(5);
        registry.Contains("food").Should().BeTrue();
        registry.TryGetResource("WOOD", out ResourceDefinition? wood).Should().BeTrue();
        wood.Should().NotBeNull();
        wood!.Name.Should().Be("Wood");

        registry.Register(new ResourceDefinition("spice", "Spice", "", 0f, 1f, 0f, true));
        registry.Contains("spice").Should().BeTrue();
        registry.GetResource("spice").Name.Should().Be("Spice");
        registry.Unregister("spice").Should().BeTrue();
    }

    [Fact]
    public void TradeRouteRegistry_CoversFilteringAndRegistration()
    {
        TradeRouteRegistry registry = new();
        registry.Register(new TradeRouteDefinition("route-wood-gold", "Wood to Gold", "wood", "gold", 10f, 30, 100f, true));
        registry.Register(new TradeRouteDefinition("route-iron-gold", "Iron to Gold", "iron", "gold", 4f, 30, 50f, false));

        registry.Count.Should().Be(2);
        registry.GetRoutesBySource("WOOD").Should().ContainSingle(r => r.Id == "route-wood-gold");
        registry.GetRoutesByTarget("gold").Should().HaveCount(2);
        registry.Enabled.Should().ContainSingle(r => r.Id == "route-wood-gold");
        registry.TryGetRoute("route-iron-gold", out TradeRouteDefinition? route).Should().BeTrue();
        route.Should().NotBeNull();
        registry.Unregister("route-iron-gold").Should().BeTrue();
    }

    [Fact]
    public void EconomyProfileRegistry_CoversRegistrationAndLookup()
    {
        EconomyProfileRegistry registry = new();
        EconomyProfile profile = new() { Id = "player", DisplayName = "Player" };

        registry.Register(profile);
        registry.Count.Should().Be(1);
        registry.Contains("PLAYER").Should().BeTrue();
        registry.GetProfile("player").DisplayName.Should().Be("Player");
        registry.TryGetProfile("player", out EconomyProfile? found).Should().BeTrue();
        found.Should().NotBeNull();
        registry.Unregister("player").Should().BeTrue();
    }

    [Fact]
    public void EconomyProfile_MultiplierFallbacksWork()
    {
        EconomyProfile profile = new();
        profile.GetProductionMultiplier("food").Should().Be(1.0f);
        profile.GetConsumptionMultiplier("food").Should().Be(1.0f);
        profile.ProductionMultipliers["food"] = 1.5f;
        profile.ConsumptionMultipliers["food"] = 0.75f;
        profile.GetProductionMultiplier("food").Should().Be(1.5f);
        profile.GetConsumptionMultiplier("food").Should().Be(0.75f);
    }

    [Fact]
    public void ThemeColorPalette_CoversLookupContrastAndColorHelpers()
    {
        ThemeThemeColorPalette palette = new()
        {
            Text = new ThemeThemeColor { R = 255, G = 255, B = 255 },
            Background = new ThemeThemeColor { R = 0, G = 0, B = 0 },
            Primary = new ThemeThemeColor { R = 32, G = 32, B = 32 },
            Secondary = new ThemeThemeColor { R = 48, G = 48, B = 48 },
            Hover = new ThemeThemeColor { R = 16, G = 16, B = 16 }
        };

        palette.GetThemeColor("primary").Should().NotBeNull();
        palette.GetThemeColor("secondary").Should().NotBeNull();
        palette.ValidateContrast().Should().BeEmpty();
        ThemeThemeColorPalette.CalculateContrastRatio(palette.Text!, palette.Background!).Should().BeGreaterThan(4.5);

        Action unknown = () => palette.GetThemeColor("not-real");
        unknown.Should().Throw<ArgumentException>();

        ThemeThemeColor parsed = ThemeThemeColor.FromHex("#1A3A6B");
        parsed.ToHex().Should().Be("#1A3A6B");
        parsed.Brighten().Should().NotBeNull();
        parsed.Darken().Should().NotBeNull();
    }

    [Fact]
    public void ThemeRegistry_CoversDefaultAndActiveThemeManagement()
    {
        ThemeRegistry registry = new();
        registry.Count.Should().Be(2);
        registry.Contains("dark-theme").Should().BeTrue();
        registry.ActiveThemeId.Should().Be("dark-theme");
        registry.ActiveTheme.Should().NotBeNull();

        registry.SetActiveTheme("light-theme").Should().BeTrue();
        registry.ActiveThemeId.Should().Be("light-theme");

        registry.Unregister("light-theme").Should().BeFalse();
        registry.Register(new ThemeDefinition("custom-theme", "Custom", "#FFFFFF", "#666666", "#FF6B00"));
        registry.Unregister("custom-theme").Should().BeTrue();
        registry.ActiveThemeId = "missing";
        registry.ActiveTheme.Should().BeNull();
        registry.SetActiveTheme("missing").Should().BeFalse();
    }

    [Fact]
    public void HudElementRegistry_CoversTypeAndVisibilityQueries()
    {
        HudElementRegistry registry = new();
        HudElementDefinition hud = new("health", "health_bar", "top_left", 320, 64)
        {
            VisibleIn = new List<string> { "gameplay", "pause" }
        };
        registry.Register(hud);

        registry.Count.Should().Be(1);
        registry.GetElementsByType("HEALTH_BAR").Should().ContainSingle();
        registry.GetElementsByVisibility("pause").Should().ContainSingle();
        registry.Unregister("health").Should().BeTrue();
    }

    [Fact]
    public void MenuRegistry_CoversHierarchyValidation()
    {
        MenuRegistry registry = new();
        MenuDefinition root = new("root", "Root");
        MenuDefinition child = new("child", "Child", "root");
        MenuDefinition orphan = new("orphan", "Orphan", "missing");
        MenuDefinition cycleA = new("cycle-a", "Cycle A", "cycle-b");
        MenuDefinition cycleB = new("cycle-b", "Cycle B", "cycle-a");

        registry.Register(root);
        registry.Register(child);
        registry.Register(orphan);
        registry.Register(cycleA);
        registry.Register(cycleB);

        registry.GetRootMenus().Should().ContainSingle(m => m.Id == "root");
        registry.GetChildMenus("root").Should().ContainSingle(m => m.Id == "child");
        IReadOnlyList<string> errors = registry.ValidateHierarchy();
        errors.Should().Contain(e => e.Contains("non-existent parent menu"));
        errors.Should().Contain(e => e.Contains("cycle"));
    }
}
