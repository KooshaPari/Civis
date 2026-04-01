#nullable enable
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using DINOForge.Domains.Economy;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Registries;
using DINOForge.Domains.Economy.Validation;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Coverage tests for Economy domain.
/// Targets: EconomyContentLoader, TradeRouteRegistry, ResourceRegistry, EconomyProfileRegistry edge cases.
/// </summary>
public class EconomyCoverageTests : IDisposable
{
    // ───────────────── EconomyContentLoader ─────────────────

    [Fact]
    public void EconomyContentLoader_LoadPack_NonExistentDirectory_ThrowsDirectoryNotFoundException()
    {
        var loader = CreateLoader();

        Action action = () => loader.LoadPack(@"C:\NonExistent\Path\Nowhere", "test-pack");

        action.Should().Throw<DirectoryNotFoundException>();
    }

    [Fact]
    public void EconomyContentLoader_LoadPack_EmptyDirectory_SucceedsWithoutThrowing()
    {
        using var tempDir = new TempDirectory();
        var loader = CreateLoader();

        Action action = () => loader.LoadPack(tempDir.Path, "test-pack");

        action.Should().NotThrow();
    }

    [Fact]
    public void EconomyContentLoader_LoadResources_WithInvalidYaml_ThrowsInvalidOperationException()
    {
        using var tempDir = new TempDirectory();
        var resourcesDir = Path.Combine(tempDir.Path, "resources");
        Directory.CreateDirectory(resourcesDir);
        File.WriteAllText(Path.Combine(resourcesDir, "bad.yaml"), "id: bad_resource\nname: [invalid: {{{");

        var loader = CreateLoader();

        Action action = () => loader.LoadPack(tempDir.Path, "test-pack");

        action.Should().Throw<InvalidOperationException>()
            .WithMessage("*resource*");
    }

    [Fact]
    public void EconomyContentLoader_LoadTradeRoutes_WithInvalidYaml_ThrowsInvalidOperationException()
    {
        using var tempDir = new TempDirectory();
        var routesDir = Path.Combine(tempDir.Path, "trade_routes");
        Directory.CreateDirectory(routesDir);
        File.WriteAllText(Path.Combine(routesDir, "routes.yaml"), "routes:\n  - id: bad\n    invalid: [");

        var loader = CreateLoader();

        Action action = () => loader.LoadPack(tempDir.Path, "test-pack");

        action.Should().Throw<InvalidOperationException>()
            .WithMessage("*trade*");
    }

    [Fact]
    public void EconomyContentLoader_LoadProfiles_WithInvalidYaml_ThrowsInvalidOperationException()
    {
        using var tempDir = new TempDirectory();
        var profilesDir = Path.Combine(tempDir.Path, "economy_profiles");
        Directory.CreateDirectory(profilesDir);
        File.WriteAllText(Path.Combine(profilesDir, "profile.yaml"), "id: bad\nproduction_multipliers:\n  stone: [invalid");

        var loader = CreateLoader();

        Action action = () => loader.LoadPack(tempDir.Path, "test-pack");

        action.Should().Throw<InvalidOperationException>()
            .WithMessage("*economy*");
    }

    [Fact]
    public void EconomyContentLoader_WithNullResourceRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new EconomyContentLoader(null!, new TradeRouteRegistry(), new EconomyProfileRegistry());

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyContentLoader_WithNullTradeRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new EconomyContentLoader(new ResourceRegistry(), null!, new EconomyProfileRegistry());

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyContentLoader_WithNullProfileRegistry_ThrowsArgumentNullException()
    {
        Action action = () => new EconomyContentLoader(new ResourceRegistry(), new TradeRouteRegistry(), null!);

        action.Should().Throw<ArgumentNullException>();
    }

    // ───────────────── TradeRouteRegistry edge cases ─────────────────

    [Fact]
    public void TradeRouteRegistry_Register_DuplicateId_Overwrites()
    {
        var registry = new TradeRouteRegistry();
        var route1 = new TradeRouteDefinition("dup_route", "Route 1", "stone", "gold", 10f, 60, 1000f, true);
        var route2 = new TradeRouteDefinition("dup_route", "Route 2", "food", "iron", 20f, 30, 500f, true);

        registry.Register(route1);
        registry.Register(route2);

        registry.Contains("dup_route").Should().BeTrue();
        registry.GetRoute("dup_route").DisplayName.Should().Be("Route 2");
    }

    [Fact]
    public void TradeRouteRegistry_GetRoute_NonExistent_ThrowsKeyNotFoundException()
    {
        var registry = new TradeRouteRegistry();

        Action action = () => registry.GetRoute("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void TradeRouteRegistry_TryGetRoute_NonExistent_ReturnsFalse()
    {
        var registry = new TradeRouteRegistry();

        bool found = registry.TryGetRoute("nonexistent", out var route);

        found.Should().BeFalse();
        route.Should().BeNull();
    }

    [Fact]
    public void TradeRouteRegistry_All_EmptyAtStart()
    {
        var registry = new TradeRouteRegistry();

        registry.All.Should().BeEmpty();
    }

    [Fact]
    public void TradeRouteRegistry_Register_Null_ThrowsArgumentNullException()
    {
        var registry = new TradeRouteRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void TradeRouteRegistry_Register_EmptyId_ThrowsArgumentException()
    {
        var registry = new TradeRouteRegistry();
        var route = new TradeRouteDefinition { Id = "  ", DisplayName = "Bad", SourceResource = "a", TargetResource = "b" };

        Action action = () => registry.Register(route);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void TradeRouteRegistry_Unregister_Existing_ReturnsTrue()
    {
        var registry = new TradeRouteRegistry();
        registry.Register(new TradeRouteDefinition("to_remove", "Remove", "stone", "gold", 10f, 60, 1000f, true));

        bool removed = registry.Unregister("to_remove");

        removed.Should().BeTrue();
        registry.Contains("to_remove").Should().BeFalse();
    }

    [Fact]
    public void TradeRouteRegistry_Unregister_NonExistent_ReturnsFalse()
    {
        var registry = new TradeRouteRegistry();

        bool removed = registry.Unregister("nonexistent");

        removed.Should().BeFalse();
    }

    [Fact]
    public void TradeRouteRegistry_GetRoutesBySource_ReturnsMatchingRoutes()
    {
        var registry = new TradeRouteRegistry();
        registry.Register(new TradeRouteDefinition("r1", "R1", "stone", "gold", 10f, 60, 1000f, true));
        registry.Register(new TradeRouteDefinition("r2", "R2", "food", "gold", 5f, 60, 500f, true));
        registry.Register(new TradeRouteDefinition("r3", "R3", "stone", "iron", 8f, 60, 800f, true));

        var stoneRoutes = registry.GetRoutesBySource("stone");

        stoneRoutes.Should().HaveCount(2);
        stoneRoutes.Should().Contain(r => r.Id == "r1");
        stoneRoutes.Should().Contain(r => r.Id == "r3");
    }

    [Fact]
    public void TradeRouteRegistry_GetRoutesByTarget_ReturnsMatchingRoutes()
    {
        var registry = new TradeRouteRegistry();
        registry.Register(new TradeRouteDefinition("r1", "R1", "stone", "gold", 10f, 60, 1000f, true));
        registry.Register(new TradeRouteDefinition("r2", "R2", "wood", "stone", 5f, 60, 500f, true));

        var goldRoutes = registry.GetRoutesByTarget("gold");

        goldRoutes.Should().HaveCount(1);
        goldRoutes[0].Id.Should().Be("r1");
    }

    // ───────────────── ResourceRegistry edge cases ─────────────────

    [Fact]
    public void ResourceRegistry_Register_Duplicate_Overwrites()
    {
        var registry = new ResourceRegistry();
        var r1 = new ResourceDefinition("dup", "Dup 1", "desc", 1.0f, 100f, 0f, true);
        var r2 = new ResourceDefinition("dup", "Dup 2", "desc2", 2.0f, 200f, 0f, true);

        registry.Register(r1);
        registry.Register(r2);

        registry.Contains("dup").Should().BeTrue();
        registry.GetResource("dup").Name.Should().Be("Dup 2");
    }

    [Fact]
    public void ResourceRegistry_GetResource_NonExistent_ThrowsKeyNotFoundException()
    {
        var registry = new ResourceRegistry();

        Action action = () => registry.GetResource("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void ResourceRegistry_TryGetResource_NonExistent_ReturnsFalse()
    {
        var registry = new ResourceRegistry();

        bool found = registry.TryGetResource("nonexistent", out var resource);

        found.Should().BeFalse();
        resource.Should().BeNull();
    }

    [Fact]
    public void ResourceRegistry_All_AfterRegistration_ContainsEntry()
    {
        var registry = new ResourceRegistry();
        registry.Register(new ResourceDefinition("test_res", "Test", "desc", 1.0f, 50f, 0f, true));

        registry.All.Should().Contain(r => r.Id == "test_res");
    }

    [Fact]
    public void ResourceRegistry_Register_Null_ThrowsArgumentNullException()
    {
        var registry = new ResourceRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ResourceRegistry_Register_EmptyId_ThrowsArgumentException()
    {
        var registry = new ResourceRegistry();
        var r = new ResourceDefinition("  ", "Bad", "desc", 1.0f, 100f, 0f, true);

        Action action = () => registry.Register(r);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void ResourceRegistry_Unregister_Existing_ReturnsTrue()
    {
        var registry = new ResourceRegistry();
        registry.Register(new ResourceDefinition("to_remove", "Remove", "desc", 1.0f, 100f, 0f, true));

        bool removed = registry.Unregister("to_remove");

        removed.Should().BeTrue();
        registry.Contains("to_remove").Should().BeFalse();
    }

    [Fact]
    public void ResourceRegistry_Unregister_NonExistent_ReturnsFalse()
    {
        var registry = new ResourceRegistry();

        bool removed = registry.Unregister("nonexistent");

        removed.Should().BeFalse();
    }

    // ───────────────── EconomyProfileRegistry edge cases ─────────────────

    [Fact]
    public void EconomyProfileRegistry_GetProfile_NonExistent_ThrowsKeyNotFoundException()
    {
        var registry = new EconomyProfileRegistry();

        Action action = () => registry.GetProfile("nonexistent");

        action.Should().Throw<KeyNotFoundException>();
    }

    [Fact]
    public void EconomyProfileRegistry_TryGetProfile_NonExistent_ReturnsFalse()
    {
        var registry = new EconomyProfileRegistry();

        bool found = registry.TryGetProfile("nonexistent", out var profile);

        found.Should().BeFalse();
        profile.Should().BeNull();
    }

    [Fact]
    public void EconomyProfileRegistry_All_EmptyAtStart()
    {
        var registry = new EconomyProfileRegistry();

        registry.All.Should().BeEmpty();
    }

    [Fact]
    public void EconomyProfileRegistry_Register_Duplicate_Overwrites()
    {
        var registry = new EconomyProfileRegistry();
        var p1 = new EconomyProfile { Id = "dup_profile", DisplayName = "Dup 1" };
        var p2 = new EconomyProfile { Id = "dup_profile", DisplayName = "Dup 2" };

        registry.Register(p1);
        registry.Register(p2);

        registry.Contains("dup_profile").Should().BeTrue();
        registry.GetProfile("dup_profile").DisplayName.Should().Be("Dup 2");
    }

    [Fact]
    public void EconomyProfileRegistry_Register_Null_ThrowsArgumentNullException()
    {
        var registry = new EconomyProfileRegistry();

        Action action = () => registry.Register(null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyProfileRegistry_Register_EmptyId_ThrowsArgumentException()
    {
        var registry = new EconomyProfileRegistry();
        var p = new EconomyProfile { Id = "  ", DisplayName = "Bad" };

        Action action = () => registry.Register(p);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void EconomyProfileRegistry_Unregister_Existing_ReturnsTrue()
    {
        var registry = new EconomyProfileRegistry();
        registry.Register(new EconomyProfile { Id = "to_remove", DisplayName = "Remove" });

        bool removed = registry.Unregister("to_remove");

        removed.Should().BeTrue();
        registry.Contains("to_remove").Should().BeFalse();
    }

    [Fact]
    public void EconomyProfileRegistry_Unregister_NonExistent_ReturnsFalse()
    {
        var registry = new EconomyProfileRegistry();

        bool removed = registry.Unregister("nonexistent");

        removed.Should().BeFalse();
    }

    [Fact]
    public void EconomyProfile_GetProductionMultiplier_UnknownResource_ReturnsDefault()
    {
        var profile = new EconomyProfile();

        float multiplier = profile.GetProductionMultiplier("unknown_resource");

        multiplier.Should().Be(1.0f);
    }

    [Fact]
    public void EconomyProfile_GetProductionMultiplier_KnownResource_ReturnsStoredValue()
    {
        var profile = new EconomyProfile();
        profile.ProductionMultipliers["stone"] = 2.5f;

        float multiplier = profile.GetProductionMultiplier("stone");

        multiplier.Should().Be(2.5f);
    }

    [Fact]
    public void EconomyProfile_GetConsumptionMultiplier_UnknownResource_ReturnsDefault()
    {
        var profile = new EconomyProfile();

        float multiplier = profile.GetConsumptionMultiplier("food");

        multiplier.Should().Be(1.0f);
    }

    [Fact]
    public void EconomyProfile_GetConsumptionMultiplier_KnownResource_ReturnsStoredValue()
    {
        var profile = new EconomyProfile();
        profile.ConsumptionMultipliers["food"] = 0.5f;

        float multiplier = profile.GetConsumptionMultiplier("food");

        multiplier.Should().Be(0.5f);
    }

    // ───────────────── TradeRouteDefinition ─────────────────

    [Fact]
    public void TradeRouteDefinition_DefaultConstructor_HasExpectedDefaults()
    {
        var route = new TradeRouteDefinition();

        route.Id.Should().BeEmpty();
        route.DisplayName.Should().BeEmpty();
        route.SourceResource.Should().BeEmpty();
        route.TargetResource.Should().BeEmpty();
        route.ExchangeRate.Should().Be(1.0f);
        route.CooldownTicks.Should().Be(60);
        route.MaxPerTransaction.Should().Be(1000.0f);
        route.Enabled.Should().BeTrue();
    }

    [Fact]
    public void TradeRouteDefinition_ParameterizedConstructor_SetsAllProperties()
    {
        var route = new TradeRouteDefinition("route1", "Trade Route 1", "stone", "gold", 15f, 120, 2000f, false);

        route.Id.Should().Be("route1");
        route.DisplayName.Should().Be("Trade Route 1");
        route.SourceResource.Should().Be("stone");
        route.TargetResource.Should().Be("gold");
        route.ExchangeRate.Should().Be(15f);
        route.CooldownTicks.Should().Be(120);
        route.MaxPerTransaction.Should().Be(2000f);
        route.Enabled.Should().BeFalse();
    }

    [Fact]
    public void TradeRouteDefinition_ParameterizedConstructor_NullId_ThrowsArgumentNullException()
    {
        Action action = () => new TradeRouteDefinition(null!, "name", "src", "tgt", 1f, 60, 1000f, true);

        action.Should().Throw<ArgumentNullException>();
    }

    // ───────────────── EconomyValidator ─────────────────

    [Fact]
    public void EconomyValidator_Validate_WithEmptyPackId_ThrowsArgumentException()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>();

        Action action = () => validator.Validate("", registries, profiles, routes);

        action.Should().Throw<ArgumentException>();
    }

    [Fact]
    public void EconomyValidator_Validate_WithNullRegistries_ThrowsArgumentNullException()
    {
        var validator = new EconomyValidator();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>();

        Action action = () => validator.Validate("test-pack", null!, profiles, routes);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyValidator_Validate_WithNullProfiles_ThrowsArgumentNullException()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var routes = new List<TradeRoute>();

        Action action = () => validator.Validate("test-pack", registries, null!, routes);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyValidator_Validate_WithNullRoutes_ThrowsArgumentNullException()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();

        Action action = () => validator.Validate("test-pack", registries, profiles, null!);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EconomyValidator_Validate_ValidInput_ReturnsNoErrors()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>
        {
            new EconomyProfile { Id = "default", TradeRateModifier = 1.0f, WorkerEfficiency = 1.0f, StorageMultiplier = 1.0f }
        };
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "stone", TargetResource = "gold", ExchangeRate = 1.5f, CooldownTicks = 60, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void EconomyValidator_Validate_DuplicateProfileId_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>
        {
            new EconomyProfile { Id = "dup", TradeRateModifier = 1.0f },
            new EconomyProfile { Id = "dup", TradeRateModifier = 1.0f }
        };
        var routes = new List<TradeRoute>();

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("Duplicate economy profile ID"));
    }

    [Fact]
    public void EconomyValidator_Validate_EmptyProfileId_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>
        {
            new EconomyProfile { Id = "", TradeRateModifier = 1.0f }
        };
        var routes = new List<TradeRoute>();

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("empty or missing ID"));
    }

    [Fact]
    public void EconomyValidator_Validate_NegativeStartingResources_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>
        {
            new EconomyProfile { Id = "bad", StartingResources = new ResourceCost { Food = -10 }, TradeRateModifier = 1.0f }
        };
        var routes = new List<TradeRoute>();

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("negative starting food"));
    }

    [Fact]
    public void EconomyValidator_Validate_DuplicateTradeRouteId_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "dup", SourceResource = "stone", TargetResource = "gold", ExchangeRate = 1.5f, CooldownTicks = 60, Enabled = true },
            new TradeRoute { Id = "dup", SourceResource = "food", TargetResource = "iron", ExchangeRate = 2.0f, CooldownTicks = 60, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("Duplicate trade route ID"));
    }

    [Fact]
    public void EconomyValidator_Validate_InvalidSourceResource_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "invalid_res", TargetResource = "gold", ExchangeRate = 1.5f, CooldownTicks = 60, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("invalid source resource"));
    }

    [Fact]
    public void EconomyValidator_Validate_SameSourceAndTargetResource_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "stone", TargetResource = "stone", ExchangeRate = 1.5f, CooldownTicks = 60, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("same source and target resource"));
    }

    [Fact]
    public void EconomyValidator_Validate_NonPositiveExchangeRate_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "stone", TargetResource = "gold", ExchangeRate = 0, CooldownTicks = 60, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("non-positive exchange rate"));
    }

    [Fact]
    public void EconomyValidator_Validate_NegativeCooldown_AddsError()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "stone", TargetResource = "gold", ExchangeRate = 1.5f, CooldownTicks = -1, Enabled = true }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("negative cooldown"));
    }

    [Fact]
    public void EconomyValidator_Validate_DisabledRoute_StillValidatedForFormat()
    {
        var validator = new EconomyValidator();
        var registries = new DINOForge.SDK.Registry.RegistryManager();
        var profiles = new List<EconomyProfile>();
        var routes = new List<TradeRoute>
        {
            new TradeRoute { Id = "route1", SourceResource = "invalid", TargetResource = "gold", ExchangeRate = 1.5f, CooldownTicks = 60, Enabled = false }
        };

        var result = validator.Validate("test-pack", registries, profiles, routes);

        // Disabled routes are still validated for format errors (invalid resources)
        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Contains("invalid source resource"));
    }

    // ───────────────── EconomyProfile property tests ─────────────────

    [Fact]
    public void EconomyProfile_DefaultValues()
    {
        var profile = new EconomyProfile();

        profile.Id.Should().BeEmpty();
        profile.DisplayName.Should().BeEmpty();
        profile.TradeRateModifier.Should().Be(1.0f);
        profile.WorkerEfficiency.Should().Be(1.0f);
        profile.StorageMultiplier.Should().Be(1.0f);
        profile.StartingResources.Should().NotBeNull();
        profile.ProductionMultipliers.Should().NotBeNull();
        profile.ConsumptionMultipliers.Should().NotBeNull();
    }

    [Fact]
    public void EconomyProfile_CanSetAllProperties()
    {
        var profile = new EconomyProfile
        {
            Id = "test-profile",
            DisplayName = "Test Profile",
            TradeRateModifier = 2.5f,
            WorkerEfficiency = 0.8f,
            StorageMultiplier = 1.5f
        };

        profile.Id.Should().Be("test-profile");
        profile.DisplayName.Should().Be("Test Profile");
        profile.TradeRateModifier.Should().Be(2.5f);
        profile.WorkerEfficiency.Should().Be(0.8f);
        profile.StorageMultiplier.Should().Be(1.5f);
    }

    [Fact]
    public void EconomyProfile_StartingResourcesCanBeModified()
    {
        var profile = new EconomyProfile();
        profile.StartingResources.Food = 100;
        profile.StartingResources.Wood = 200;

        profile.StartingResources.Food.Should().Be(100);
        profile.StartingResources.Wood.Should().Be(200);
    }

    // ───────────────── TradeRoute property tests ─────────────────

    [Fact]
    public void TradeRoute_DefaultValues()
    {
        var route = new TradeRoute();

        route.Id.Should().BeEmpty();
        route.DisplayName.Should().BeEmpty();
        route.SourceResource.Should().BeEmpty();
        route.TargetResource.Should().BeEmpty();
        route.ExchangeRate.Should().Be(1.0f);
        route.CooldownTicks.Should().Be(60);
        route.MaxPerTransaction.Should().Be(0);
        route.Enabled.Should().BeTrue();
    }

    [Fact]
    public void TradeRoute_CanSetAllProperties()
    {
        var route = new TradeRoute
        {
            Id = "test-route",
            DisplayName = "Test Route",
            SourceResource = "iron",
            TargetResource = "gold",
            ExchangeRate = 3.0f,
            CooldownTicks = 120,
            MaxPerTransaction = 500,
            Enabled = true
        };

        route.Id.Should().Be("test-route");
        route.DisplayName.Should().Be("Test Route");
        route.SourceResource.Should().Be("iron");
        route.TargetResource.Should().Be("gold");
        route.ExchangeRate.Should().Be(3.0f);
        route.CooldownTicks.Should().Be(120);
        route.MaxPerTransaction.Should().Be(500);
        route.Enabled.Should().BeTrue();
    }

    // ───────────────── ResourceCost tests ─────────────────

    [Fact]
    public void ResourceCost_DefaultValues()
    {
        var cost = new ResourceCost();

        cost.Food.Should().Be(0);
        cost.Wood.Should().Be(0);
        cost.Stone.Should().Be(0);
        cost.Iron.Should().Be(0);
        cost.Gold.Should().Be(0);
    }

    [Fact]
    public void ResourceCost_CanSetAllProperties()
    {
        var cost = new ResourceCost
        {
            Food = 100,
            Wood = 200,
            Stone = 300,
            Iron = 400,
            Gold = 500
        };

        cost.Food.Should().Be(100);
        cost.Wood.Should().Be(200);
        cost.Stone.Should().Be(300);
        cost.Iron.Should().Be(400);
        cost.Gold.Should().Be(500);
    }

    // ───────────────── Helper ─────────────────

    private static EconomyContentLoader CreateLoader()
    {
        return new EconomyContentLoader(
            new ResourceRegistry(),
            new TradeRouteRegistry(),
            new EconomyProfileRegistry());
    }

    public void Dispose()
    {
        // Cleanup if needed
    }

    /// <summary>RAII temp directory that auto-deletes on dispose.</summary>
    private sealed class TempDirectory : IDisposable
    {
        public string Path { get; }

        public TempDirectory()
        {
            Path = System.IO.Path.Combine(System.IO.Path.GetTempPath(), $"dinotest_{Guid.NewGuid():N}");
            Directory.CreateDirectory(Path);
        }

        public void Dispose()
        {
            try { Directory.Delete(Path, recursive: true); } catch { }
        }
    }
}
