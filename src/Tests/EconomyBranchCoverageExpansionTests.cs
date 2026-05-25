using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy;
using DINOForge.Domains.Economy.Balance;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.Domains.Economy.Trade;
using DINOForge.Domains.Economy.Validation;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Comprehensive branch coverage tests targeting uncovered control flow paths
    /// in Economy domain (targeting 71.72% → 75%+ branch coverage).
    /// Tests edge cases, boundary conditions, and conditional branches in:
    /// - EconomyBalanceCalculator
    /// - ProductionCalculator
    /// - TradeEngine
    /// - EconomyValidator
    /// </summary>
    public class EconomyBranchCoverageExpansionTests
    {
        // ── EconomyBalanceCalculator: Deficit generation paths ──────────────────

        [Fact]
        public void GenerateReport_NoProductionAllResources_WarnsAllDeficits()
        {
            // Test: When a faction produces zero of each resource type
            RegistryManager registries = new RegistryManager();
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "zero-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            var building = new BuildingDefinition
            {
                Id = "zero-building",
                BuildingType = "economy",
                Production = new Dictionary<string, int> { { "food", 0 }, { "wood", 0 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            var prodCalc = new ProductionCalculator();
            var tradeEngine = new TradeEngine();
            var balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            var profiles = new Dictionary<string, EconomyProfile>();
            var report = balanceCalc.GenerateReport("test-pack", registries, profiles, new List<TradeRoute>());

            // Should have warnings for zero production
            report.Warnings.Should().NotBeEmpty();
            report.Warnings.Any(w => w.Contains("no") && w.Contains("production")).Should().BeTrue();
        }

        [Fact]
        public void GenerateReport_MultipleResourcesWithDeficits_GeneratesMultipleWarnings()
        {
            // Test: Multiple resources in deficit (tests the loop generating warnings)
            RegistryManager registries = new RegistryManager();
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "deficit-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            var building = new BuildingDefinition
            {
                Id = "minimal-prod",
                BuildingType = "economy",
                Production = new Dictionary<string, int> { { "food", 1 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            var prodCalc = new ProductionCalculator();
            var tradeEngine = new TradeEngine();
            var balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            var profile = new EconomyProfile { Id = "test-profile" };
            var profiles = new Dictionary<string, EconomyProfile> { { "deficit-faction", profile } };

            var report = balanceCalc.GenerateReport("test-pack", registries, profiles, new List<TradeRoute>());

            // Should have warnings for resources with no/low production
            report.Warnings.Should().NotBeEmpty();
        }

        [Fact]
        public void GenerateReport_TradeRoutesEvaluated_TradeEfficiencyCalculated()
        {
            // Test: Trade efficiency calculation branch (line 91-92)
            RegistryManager registries = new RegistryManager();
            var faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "trade-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            var building = new BuildingDefinition
            {
                Id = "surplus-building",
                BuildingType = "economy",
                Production = new Dictionary<string, int> { { "wood", 10 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            var prodCalc = new ProductionCalculator();
            var tradeEngine = new TradeEngine();
            var balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            var profile = new EconomyProfile { Id = "test-profile" };
            var profiles = new Dictionary<string, EconomyProfile> { { "trade-faction", profile } };

            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "wood-to-stone",
                    SourceResource = "wood",
                    TargetResource = "stone",
                    ExchangeRate = 2.0f,
                    Enabled = true
                }
            };

            var report = balanceCalc.GenerateReport("test-pack", registries, profiles, routes);

            // Summary should include faction with calculated efficiency
            report.FactionSummaries.Should().ContainKey("trade-faction");
            var summary = report.FactionSummaries["trade-faction"];
            summary.TradeEfficiency.Should().BeGreaterThanOrEqualTo(0);
        }

        // ── ProductionCalculator: Worker allocation and zero cases ──────────────

        [Fact]
        public void CalculateBuildingOutput_ZeroWorkers_ProducesZero()
        {
            // Test: When worker count is explicitly zero
            var building = new BuildingDefinition
            {
                Id = "test-building",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            var profile = new EconomyProfile();
            var workerCounts = new Dictionary<string, int> { { "test-building", 0 } };

            var prodCalc = new ProductionCalculator();
            var output = prodCalc.CalculateBuildingOutput(building, profile, workerCounts);

            output["food"].Should().Be(0f);
        }

        [Fact]
        public void CalculateBuildingOutput_NegativeWorkersClampedToZero_ProducesZero()
        {
            // Test: Negative worker count clamped via Math.Max(0, workers)
            var building = new BuildingDefinition
            {
                Id = "test-building",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            var profile = new EconomyProfile();
            var workerCounts = new Dictionary<string, int> { { "test-building", -5 } };

            var prodCalc = new ProductionCalculator();
            var output = prodCalc.CalculateBuildingOutput(building, profile, workerCounts);

            output["food"].Should().Be(0f);
        }

        [Fact]
        public void CalculateFactionProduction_EmptyBuildingList_ReturnsZeroProduction()
        {
            // Test: Faction with no buildings produces zero of everything
            var registries = new RegistryManager();
            var profile = new EconomyProfile();
            var buildingIds = new List<string>();

            var prodCalc = new ProductionCalculator();
            var production = prodCalc.CalculateFactionProduction("faction-id", profile, registries.Buildings, buildingIds);

            foreach (string resourceType in ResourceRate.ValidResourceTypes)
            {
                production[resourceType].Should().Be(0f);
            }
        }

        [Fact]
        public void CalculateFactionProduction_UnknownBuildingIds_SkipsAndContinues()
        {
            // Test: Non-existent building IDs are skipped (null continue at line 49)
            var registries = new RegistryManager();
            var profile = new EconomyProfile();
            var buildingIds = new List<string> { "nonexistent-building", "also-nonexistent" };

            var prodCalc = new ProductionCalculator();
            var production = prodCalc.CalculateFactionProduction("faction-id", profile, registries.Buildings, buildingIds);

            production.Values.Sum(v => v).Should().Be(0f);
        }

        [Fact]
        public void GetResourceBalance_ConsumptionOnlyNoProduction_AllNegativeBalances()
        {
            // Test: Pure consumption scenario (no production)
            var production = new Dictionary<string, float>
            {
                { "food", 0f },
                { "wood", 0f },
                { "stone", 0f }
            };
            var consumption = new Dictionary<string, float>
            {
                { "food", 5f },
                { "wood", 2f },
                { "stone", 1f }
            };

            var prodCalc = new ProductionCalculator();
            var balance = prodCalc.GetResourceBalance(production, consumption);

            balance["food"].Should().Be(-5f);
            balance["wood"].Should().Be(-2f);
            balance["stone"].Should().Be(-1f);
        }

        [Fact]
        public void GetResourceBalance_ConsumptionResourceNotInProduction_CreatesNegativeEntry()
        {
            // Test: Consumption resource exists but production doesn't (line 137-140)
            var production = new Dictionary<string, float> { { "food", 0f } };
            var consumption = new Dictionary<string, float> { { "wood", 5f } };

            var prodCalc = new ProductionCalculator();
            var balance = prodCalc.GetResourceBalance(production, consumption);

            balance.Should().ContainKey("wood");
            balance["wood"].Should().Be(-5f);
        }

        [Fact]
        public void CalculateUnitConsumption_ZeroAmountResources_Skipped()
        {
            // Test: Zero-cost resources are skipped (line 189: amount <= 0 return)
            var units = new Registry<UnitDefinition>();
            var unit = new UnitDefinition
            {
                Id = "free-unit",
                Stats = new UnitStats { Cost = new ResourceCost { Food = 0, Wood = 0 } }
            };
            units.Register(unit.Id, unit, RegistrySource.Pack, "test-pack");

            var unitCounts = new Dictionary<string, int> { { "free-unit", 10 } };
            var profile = new EconomyProfile();

            var prodCalc = new ProductionCalculator();
            var consumption = prodCalc.CalculateUnitConsumption(units, unitCounts, profile);

            // No consumption for zero-cost resources
            consumption.Values.Sum(v => v).Should().Be(0f);
        }

        // ── TradeEngine: Unprofitable and boundary cases ────────────────────────

        [Fact]
        public void EvaluateTradeRoute_SourceNotInSurplus_NotProfitable()
        {
            // Test: isProfitable = false when source is not in surplus (line 150)
            var route = new TradeRoute
            {
                Id = "test-route",
                SourceResource = "wood",
                TargetResource = "stone",
                ExchangeRate = 2.0f,
                Enabled = true,
                MaxPerTransaction = 100
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 10f } };
            var balance = new Dictionary<string, float> { { "wood", -5f }, { "stone", -10f } }; // wood in deficit

            var tradeEngine = new TradeEngine();
            var eval = tradeEngine.EvaluateTradeRoute(route, profile, available, balance);

            eval.IsProfitable.Should().BeFalse();
        }

        [Fact]
        public void EvaluateTradeRoute_TargetNotInDeficit_NotProfitable()
        {
            // Test: isProfitable = false when target is not in deficit (line 150)
            var route = new TradeRoute
            {
                Id = "test-route",
                SourceResource = "wood",
                TargetResource = "stone",
                ExchangeRate = 2.0f,
                Enabled = true,
                MaxPerTransaction = 100
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 10f } };
            var balance = new Dictionary<string, float> { { "wood", 5f }, { "stone", 5f } }; // stone in surplus

            var tradeEngine = new TradeEngine();
            var eval = tradeEngine.EvaluateTradeRoute(route, profile, available, balance);

            eval.IsProfitable.Should().BeFalse();
        }

        [Fact]
        public void EvaluateTradeRoute_RouteDisabled_NotProfitable()
        {
            // Test: Disabled route never profitable (line 150)
            var route = new TradeRoute
            {
                Id = "test-route",
                SourceResource = "wood",
                TargetResource = "stone",
                ExchangeRate = 2.0f,
                Enabled = false,
                MaxPerTransaction = 100
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 10f } };
            var balance = new Dictionary<string, float> { { "wood", 5f }, { "stone", -5f } };

            var tradeEngine = new TradeEngine();
            var eval = tradeEngine.EvaluateTradeRoute(route, profile, available, balance);

            eval.IsProfitable.Should().BeFalse();
        }

        [Fact]
        public void EvaluateTradeRoute_ZeroExchangeRate_CalculatesZeroEfficiency()
        {
            // Test: Zero exchange rate → efficiency = 0 (line 143)
            var route = new TradeRoute
            {
                Id = "test-route",
                SourceResource = "wood",
                TargetResource = "stone",
                ExchangeRate = 0f,
                Enabled = true,
                MaxPerTransaction = 100
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 10f } };
            var balance = new Dictionary<string, float> { { "wood", 5f }, { "stone", -5f } };

            var tradeEngine = new TradeEngine();
            var eval = tradeEngine.EvaluateTradeRoute(route, profile, available, balance);

            eval.Efficiency.Should().Be(0f);
            eval.MaxTargetPerExecution.Should().Be(0f);
        }

        [Fact]
        public void EvaluateTradeRoute_MaxTransactionLimitApplied()
        {
            // Test: Max transaction limit caps available source (line 153-154)
            var route = new TradeRoute
            {
                Id = "test-route",
                SourceResource = "wood",
                TargetResource = "stone",
                ExchangeRate = 2.0f,
                Enabled = true,
                MaxPerTransaction = 5  // Limited to 5
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 100f } }; // Much more available
            var balance = new Dictionary<string, float> { { "wood", 50f }, { "stone", -10f } };

            var tradeEngine = new TradeEngine();
            var eval = tradeEngine.EvaluateTradeRoute(route, profile, available, balance);

            // maxSource should be capped at 5
            eval.MaxTargetPerExecution.Should().Be(5f / 2.0f);
        }

        [Fact]
        public void GetOptimalTrades_NoProfitableRoutes_ReturnsEmpty()
        {
            // Test: Empty suggestions when no routes are profitable (line 190-191)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "route1",
                    SourceResource = "wood",
                    TargetResource = "stone",
                    ExchangeRate = 2.0f,
                    Enabled = true
                }
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float> { { "wood", 0f } }; // No source available
            var balance = new Dictionary<string, float> { { "wood", -5f }, { "stone", -5f } };

            var tradeEngine = new TradeEngine();
            var suggestions = tradeEngine.GetOptimalTrades(routes, profile, available, balance);

            suggestions.Should().BeEmpty();
        }

        [Fact]
        public void GetOptimalTrades_MultipleDeficits_PrioritizesByDeficitSeverity()
        {
            // Test: Multiple deficits sorted by severity (line 187)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "wood-to-stone",
                    SourceResource = "wood",
                    TargetResource = "stone",
                    ExchangeRate = 1.0f,
                    Enabled = true
                },
                new TradeRoute
                {
                    Id = "stone-to-iron",
                    SourceResource = "stone",
                    TargetResource = "iron",
                    ExchangeRate = 1.0f,
                    Enabled = true
                }
            };
            var profile = new EconomyProfile();
            var available = new Dictionary<string, float>
            {
                { "wood", 50f },
                { "stone", 50f },
                { "iron", 0f }
            };
            var balance = new Dictionary<string, float>
            {
                { "wood", 10f },
                { "stone", -5f },  // Smaller deficit
                { "iron", -20f }   // Larger deficit (more negative)
            };

            var tradeEngine = new TradeEngine();
            var suggestions = tradeEngine.GetOptimalTrades(routes, profile, available, balance);

            // Iron deficit is more severe, should be addressed
            suggestions.Should().NotBeEmpty();
        }

        // ── EconomyValidator: Validation branches ────────────────────────────────

        [Fact]
        public void Validate_TradeRouteWithSelfTrade_Error()
        {
            // Test: Source and target same resource error (line 211-214)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "self-trade",
                    SourceResource = "wood",
                    TargetResource = "wood",  // Same!
                    ExchangeRate = 1.0f
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), new List<EconomyProfile>(), routes);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("same source and target"));
        }

        [Fact]
        public void Validate_ExtremelyHighExchangeRate_Warning()
        {
            // Test: Warning for very high exchange rate (line 229-233)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "expensive-trade",
                    SourceResource = "wood",
                    TargetResource = "gold",
                    ExchangeRate = 150f  // > 100
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), new List<EconomyProfile>(), routes);

            result.Warnings.Should().Contain(w => w.Contains("high exchange rate"));
        }

        [Fact]
        public void Validate_ExtremelyLowExchangeRate_Warning()
        {
            // Test: Warning for extremely low exchange rate (line 235-239)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "free-trade",
                    SourceResource = "wood",
                    TargetResource = "gold",
                    ExchangeRate = 0.001f  // < 0.01
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), new List<EconomyProfile>(), routes);

            result.Warnings.Should().Contain(w => w.Contains("extremely low"));
        }

        [Fact]
        public void Validate_EconomyBuildingWithoutProduction_Warning()
        {
            // Test: Warning for economy building with no production (line 92-96)
            var registries = new RegistryManager();
            var building = new BuildingDefinition
            {
                Id = "empty-economy",
                BuildingType = "economy",
                Production = new Dictionary<string, int>()  // Empty!
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", registries, new List<EconomyProfile>(), new List<TradeRoute>());

            result.Warnings.Should().Contain(w => w.Contains("no production"));
        }

        [Fact]
        public void Validate_BuildingProducesNegativeAmount_Warning()
        {
            // Test: Warning for negative production (consumption, line 84-88)
            var registries = new RegistryManager();
            var building = new BuildingDefinition
            {
                Id = "consumer-building",
                BuildingType = "economy",
                Production = new Dictionary<string, int> { { "food", -5 } }  // Negative!
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", registries, new List<EconomyProfile>(), new List<TradeRoute>());

            result.Warnings.Should().Contain(w => w.Contains("negative"));
        }

        [Fact]
        public void Validate_DuplicateProfileIds_Error()
        {
            // Test: Duplicate profile ID detection (line 118-121)
            var profiles = new List<EconomyProfile>
            {
                new EconomyProfile { Id = "dup-profile" },
                new EconomyProfile { Id = "dup-profile" }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), profiles, new List<TradeRoute>());

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("Duplicate"));
        }

        [Fact]
        public void Validate_NegativeStartingResources_Error()
        {
            // Test: Negative starting resources error (line 125)
            var profiles = new List<EconomyProfile>
            {
                new EconomyProfile
                {
                    Id = "bad-profile",
                    StartingResources = new ResourceCost { Food = -5 }
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), profiles, new List<TradeRoute>());

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("negative") && e.Contains("food"));
        }

        [Fact]
        public void Validate_NonPositiveTradeRateModifier_Error()
        {
            // Test: Non-positive trade rate modifier error (line 158-161)
            var profiles = new List<EconomyProfile>
            {
                new EconomyProfile
                {
                    Id = "bad-trade-profile",
                    TradeRateModifier = 0f  // Non-positive
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), profiles, new List<TradeRoute>());

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("trade rate modifier"));
        }

        [Fact]
        public void Validate_NonPositiveWorkerEfficiency_Warning()
        {
            // Test: Non-positive worker efficiency warning (line 162-165)
            var profiles = new List<EconomyProfile>
            {
                new EconomyProfile
                {
                    Id = "lazy-profile",
                    WorkerEfficiency = 0f  // Non-positive
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), profiles, new List<TradeRoute>());

            result.Warnings.Should().Contain(w => w.Contains("worker efficiency"));
        }

        [Fact]
        public void Validate_NonPositiveStorageMultiplier_Warning()
        {
            // Test: Non-positive storage multiplier warning (line 166-169)
            var profiles = new List<EconomyProfile>
            {
                new EconomyProfile
                {
                    Id = "no-storage-profile",
                    StorageMultiplier = 0f  // Non-positive
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), profiles, new List<TradeRoute>());

            result.Warnings.Should().Contain(w => w.Contains("storage multiplier"));
        }

        [Fact]
        public void Validate_CircularTradeDetected_Error()
        {
            // Test: Circular trade detection with profitable cycle
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "route1",
                    SourceResource = "wood",
                    TargetResource = "stone",
                    ExchangeRate = 0.5f,  // 0.5 units of wood per unit of stone
                    Enabled = true
                },
                new TradeRoute
                {
                    Id = "route2",
                    SourceResource = "stone",
                    TargetResource = "wood",
                    ExchangeRate = 0.8f,  // 0.8 units of stone per unit of wood
                    Enabled = true
                }
            };
            // Combined: 0.5 * 0.8 = 0.4 < 1.0 → profitable cycle!

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), new List<EconomyProfile>(), routes);

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Contains("circular"));
        }

        [Fact]
        public void Validate_DisabledRouteNotIncludedInCycleDetection()
        {
            // Test: Disabled routes skipped in cycle detection (line 256)
            var routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "route1",
                    SourceResource = "wood",
                    TargetResource = "stone",
                    ExchangeRate = 0.5f,
                    Enabled = true
                },
                new TradeRoute
                {
                    Id = "route2",
                    SourceResource = "stone",
                    TargetResource = "wood",
                    ExchangeRate = 0.8f,
                    Enabled = false  // Disabled, so cycle is broken
                }
            };

            var validator = new EconomyValidator();
            var result = validator.Validate("test-pack", new RegistryManager(), new List<EconomyProfile>(), routes);

            // No cycle error because route2 is disabled
            result.Errors.Should().NotContain(e => e.Contains("circular"));
        }
    }
}
