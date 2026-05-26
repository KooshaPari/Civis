// Copyright (c) DINOForge Contributors. Licensed under MIT.
// Task #210 Phase 5 — EconomyContentLoader JsonGuard wiring negative tests.
// Mirrors PackLoaderTests.cs / UIContentLoaderValidationTests.cs Pattern #75 / Pattern #86 negative-test pattern.

using System;
using System.IO;
using DINOForge.Domains.Economy;
using DINOForge.Domains.Economy.Registries;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Pins the IValidatable wiring at the three EconomyContentLoader deserialize sites
    /// (resources, trade routes, economy profiles). Per Pattern #95/#210 (iter-128),
    /// registry Register() methods throw <see cref="ArgumentException"/> for validation
    /// failures. EconomyContentLoader wraps every exception in
    /// <see cref="InvalidOperationException"/>; the underlying validation fault is
    /// the <see cref="ArgumentException"/> carried as InnerException.
    ///
    /// These negative tests enforce that:
    ///   - ResourceDefinition validates id at register time
    ///   - TradeRouteDefinition validates id, source/target, and exchange rates at register time
    ///   - EconomyProfile validates id, display_name, and multipliers at register time
    /// at the deserialize site, not later.
    /// </summary>
    public class EconomyContentLoaderValidationTests : IDisposable
    {
        private readonly string _packDir;
        private readonly EconomyContentLoader _loader;
        private readonly ResourceRegistry _resourceRegistry;
        private readonly TradeRouteRegistry _tradeRouteRegistry;
        private readonly EconomyProfileRegistry _profileRegistry;

        public EconomyContentLoaderValidationTests()
        {
            _packDir = Path.Combine(
                Path.GetTempPath(),
                "dinoforge-economycontentloader-tests-" + Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(_packDir);

            _resourceRegistry = new ResourceRegistry();
            _tradeRouteRegistry = new TradeRouteRegistry();
            _profileRegistry = new EconomyProfileRegistry();
            _loader = new EconomyContentLoader(
                _resourceRegistry,
                _tradeRouteRegistry,
                _profileRegistry);
        }

        public void Dispose()
        {
            try
            {
                if (Directory.Exists(_packDir))
                {
                    Directory.Delete(_packDir, recursive: true);
                }
            }
            catch (IOException)
            {
                // Best-effort cleanup; leave the temp dir if locked by an antivirus etc.
            }
        }

        // ── ResourceDefinition validation ─────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsResourceWithBlankId()
        {
            string resourcesDir = Path.Combine(_packDir, "resources");
            Directory.CreateDirectory(resourcesDir);
            string yaml = @"
id: ''
name: Blank Id Resource
description: Has empty id
production_rate: 1.0
storage_capacity: 1000
decay_rate: 0
is_tradeable_default: true
";
            File.WriteAllText(Path.Combine(resourcesDir, "bad-resource.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-resource-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*id*");

            // ResourceRegistry pre-registers 5 canonical resources
            // (food/wood/stone/iron/gold); the bad pack must add 0 beyond those.
            _resourceRegistry.Count.Should().Be(5);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsResourceWithNegativeStorageCapacity()
        {
            string resourcesDir = Path.Combine(_packDir, "resources");
            Directory.CreateDirectory(resourcesDir);
            string yaml = @"
id: bad-storage
name: Bad Storage
description: Has negative storage_capacity
production_rate: 1.0
storage_capacity: -10
decay_rate: 0
is_tradeable_default: true
";
            File.WriteAllText(Path.Combine(resourcesDir, "bad-storage.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-storage-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*storage_capacity*");

            // ResourceRegistry pre-registers 5 canonical resources; the bad
            // pack must add 0 beyond those.
            _resourceRegistry.Count.Should().Be(5);
        }

        // ── TradeRouteDefinition validation ───────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsTradeRouteWithBlankId()
        {
            string routesDir = Path.Combine(_packDir, "trade_routes");
            Directory.CreateDirectory(routesDir);
            string yaml = @"
routes:
  - id: ''
    display_name: Blank Id Route
    source_resource: wood
    target_resource: gold
    exchange_rate: 10.0
    cooldown_ticks: 60
    max_per_transaction: 1000
    enabled: true
";
            File.WriteAllText(Path.Combine(routesDir, "bad-route.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-route-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*id*");

            _tradeRouteRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsTradeRouteWithNonPositiveExchangeRate()
        {
            string routesDir = Path.Combine(_packDir, "trade_routes");
            Directory.CreateDirectory(routesDir);
            string yaml = @"
routes:
  - id: zero-rate-route
    display_name: Zero Rate Route
    source_resource: wood
    target_resource: gold
    exchange_rate: 0
    cooldown_ticks: 60
    max_per_transaction: 1000
    enabled: true
";
            File.WriteAllText(Path.Combine(routesDir, "zero-rate.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "zero-rate-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*exchange_rate*");

            _tradeRouteRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsTradeRouteWithBlankSourceResource()
        {
            string routesDir = Path.Combine(_packDir, "trade_routes");
            Directory.CreateDirectory(routesDir);
            string yaml = @"
routes:
  - id: blank-source
    display_name: Blank Source
    source_resource: ''
    target_resource: gold
    exchange_rate: 10.0
    cooldown_ticks: 60
    max_per_transaction: 1000
    enabled: true
";
            File.WriteAllText(Path.Combine(routesDir, "blank-source.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "blank-source-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*source_resource*");

            _tradeRouteRegistry.Count.Should().Be(0);
        }

        // ── EconomyProfile validation ─────────────────────────────

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsEconomyProfileWithBlankId()
        {
            string profilesDir = Path.Combine(_packDir, "economy_profiles");
            Directory.CreateDirectory(profilesDir);
            string yaml = @"
id: ''
display_name: Blank Id Profile
production_multipliers:
  wood: 1.0
consumption_multipliers:
  wood: 1.0
trade_rate_modifier: 1.0
trade_cooldown_modifier: 1.0
storage_multiplier: 1.0
building_cost_modifier: 1.0
worker_efficiency: 1.0
";
            File.WriteAllText(Path.Combine(profilesDir, "bad-profile.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-profile-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*id*");

            _profileRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsEconomyProfileWithNegativeMultiplier()
        {
            string profilesDir = Path.Combine(_packDir, "economy_profiles");
            Directory.CreateDirectory(profilesDir);
            string yaml = @"
id: bad-modifier
display_name: Bad Modifier Profile
production_multipliers:
  wood: 1.0
consumption_multipliers:
  wood: 1.0
trade_rate_modifier: -0.5
trade_cooldown_modifier: 1.0
storage_multiplier: 1.0
building_cost_modifier: 1.0
worker_efficiency: 1.0
";
            File.WriteAllText(Path.Combine(profilesDir, "bad-modifier.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "bad-modifier-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*trade_rate_modifier*");

            _profileRegistry.Count.Should().Be(0);
        }

        [Fact]
        [Trait("Category", "Validation")]
        public void EconomyContentLoader_RejectsEconomyProfileWithBlankDisplayName()
        {
            string profilesDir = Path.Combine(_packDir, "economy_profiles");
            Directory.CreateDirectory(profilesDir);
            string yaml = @"
id: blank-display-profile
display_name: ''
production_multipliers: {}
consumption_multipliers: {}
trade_rate_modifier: 1.0
trade_cooldown_modifier: 1.0
storage_multiplier: 1.0
building_cost_modifier: 1.0
worker_efficiency: 1.0
";
            File.WriteAllText(Path.Combine(profilesDir, "blank-display.yaml"), yaml);

            Action act = () => _loader.LoadPack(_packDir, "blank-display-profile-pack");

            act.Should()
                .Throw<InvalidOperationException>()
                .WithInnerException<ArgumentException>()
                .WithMessage("*display_name*");

            _profileRegistry.Count.Should().Be(0);
        }
    }
}
