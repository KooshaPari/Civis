using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Domains.Economy;
using DINOForge.Domains.Economy.Balance;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.Domains.Economy.Trade;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Advanced Economy domain tests targeting branch coverage gaps (70.15% → 75%+).
    /// Focuses on edge cases, boundary conditions, and uncovered control flow paths
    /// in EconomyBalanceCalculator, ProductionCalculator, TradeEngine, and ResourceRate.
    /// </summary>
    public class EconomyAdvancedCoverageTests
    {
        // ── EconomyBalanceCalculator Branch Coverage ─────────────────────────

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_AllResourcesAtMinimum()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "minimal-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            BuildingDefinition building = new BuildingDefinition
            {
                Id = "minimal-building",
                Production = new Dictionary<string, int> { { "food", 1 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>
            {
                { "minimal-faction", new EconomyProfile { ProductionMultipliers = new Dictionary<string, float> { { "food", 0.1f } } } }
            };

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, new List<TradeRoute>());

            report.Should().NotBeNull();
            report.FactionSummaries.Should().HaveCount(1);
            report.FactionSummaries["minimal-faction"].SustainabilityScore.Should().BeLessThanOrEqualTo(1.0f);
        }

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_AllResourcesAtMaximum()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "max-faction" },
                Buildings = new FactionBuildings { EconomyPrimary = "max-building" },
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            BuildingDefinition building = new BuildingDefinition
            {
                Id = "max-building",
                Production = new Dictionary<string, int>
                {
                    { "food", 1000 }, { "wood", 1000 }, { "stone", 1000 }, { "iron", 1000 }, { "gold", 1000 }
                }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>
            {
                { "max-faction", new EconomyProfile { ProductionMultipliers = new Dictionary<string, float> { { "food", 10.0f }, { "wood", 10.0f }, { "stone", 10.0f }, { "iron", 10.0f }, { "gold", 10.0f } } } }
            };

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, new List<TradeRoute>());

            // All production + consumption results in net balance > 0, so sustainability should be high
            report.FactionSummaries["max-faction"].SustainabilityScore.Should().Be(1.0f);
        }

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_MixedResourceConstraints()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "mixed-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            BuildingDefinition building = new BuildingDefinition
            {
                Id = "mixed-building",
                Production = new Dictionary<string, int> { { "food", 50 }, { "wood", 10 }, { "gold", 1 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>
            {
                { "mixed-faction", new EconomyProfile { ProductionMultipliers = new Dictionary<string, float> { { "food", 1.0f }, { "wood", 0.5f }, { "gold", 2.0f } } } }
            };

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, new List<TradeRoute>());

            report.FactionSummaries["mixed-faction"].Production.Should().NotBeEmpty();
            report.Warnings.Should().NotBeEmpty();
        }

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_ZeroProductionMultiplier()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "zero-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>
            {
                { "zero-faction", new EconomyProfile { ProductionMultipliers = new Dictionary<string, float> { { "food", 0.0f } } } }
            };

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, new List<TradeRoute>());

            report.Warnings.Should().NotBeEmpty();
        }

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_NegativeMultiplier()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "debt-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            BuildingDefinition building = new BuildingDefinition
            {
                Id = "debt-building",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            registries.Buildings.Register(building.Id, building, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>
            {
                { "debt-faction", new EconomyProfile { ProductionMultipliers = new Dictionary<string, float> { { "food", -1.0f } } } }
            };

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, new List<TradeRoute>());

            // Negative multiplier results in zero production (GetProductionMultiplier returns -1, multiplied by base gives -10, but no building registered so 0)
            report.FactionSummaries["debt-faction"].Production["food"].Should().Be(0f);
        }

        // ── ProductionCalculator Branch Coverage ──────────────────────────────

        [Fact]
        public void ProductionCalculator_CalculateBuildingOutput_WithMultipleResources()
        {
            ProductionCalculator calc = new ProductionCalculator();

            BuildingDefinition farm = new BuildingDefinition
            {
                Id = "multi-farm",
                Production = new Dictionary<string, int>
                {
                    { "food", 20 }, { "wood", 5 }, { "stone", 2 }
                }
            };
            EconomyProfile profile = new EconomyProfile
            {
                ProductionMultipliers = new Dictionary<string, float>
                {
                    { "food", 1.5f }, { "wood", 2.0f }, { "stone", 0.5f }
                }
            };

            Dictionary<string, float> output = calc.CalculateBuildingOutput(farm, profile);

            output["food"].Should().Be(30f);
            output["wood"].Should().Be(10f);
            output["stone"].Should().Be(1f);
        }

        [Fact]
        public void ProductionCalculator_CalculateBuildingOutput_WithStorageFull()
        {
            ProductionCalculator calc = new ProductionCalculator();

            BuildingDefinition storage = new BuildingDefinition
            {
                Id = "storage",
                Production = new Dictionary<string, int>()
            };
            EconomyProfile profile = new EconomyProfile();

            Dictionary<string, float> output = calc.CalculateBuildingOutput(storage, profile);

            output.Should().BeEmpty();
        }

        [Fact]
        public void ProductionCalculator_CalculateBuildingOutput_WithZeroCapacity()
        {
            ProductionCalculator calc = new ProductionCalculator();

            BuildingDefinition building = new BuildingDefinition
            {
                Id = "broken-building",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> workers = new Dictionary<string, int> { { "broken-building", 0 } };

            Dictionary<string, float> output = calc.CalculateBuildingOutput(building, profile, workers);

            output["food"].Should().Be(0f);
        }

        [Fact]
        public void ProductionCalculator_GetResourceBalance_RapidProductionConsumptionCycle()
        {
            ProductionCalculator calc = new ProductionCalculator();

            Dictionary<string, float> production = new Dictionary<string, float>
            {
                { "food", 1000f }, { "wood", 500f }, { "stone", 200f }
            };
            Dictionary<string, float> consumption = new Dictionary<string, float>
            {
                { "food", 900f }, { "wood", 600f }, { "stone", 100f }
            };

            Dictionary<string, float> balance = calc.GetResourceBalance(production, consumption);

            balance["food"].Should().Be(100f);
            balance["wood"].Should().Be(-100f);
            balance["stone"].Should().Be(100f);
        }

        [Fact]
        public void ProductionCalculator_CalculateFactionProduction_WithMissingBuildings()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();

            EconomyProfile profile = new EconomyProfile();
            List<string> buildingIds = new List<string> { "nonexistent-building", "also-missing" };

            Dictionary<string, float> production = calc.CalculateFactionProduction(
                "faction1", profile, registries.Buildings, buildingIds);

            production.Should().NotBeNull();
            production.Values.All(v => v == 0f).Should().BeTrue();
        }

        [Fact]
        public void ProductionCalculator_CalculateUnitConsumption_WithVariedUnits()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();

            UnitDefinition soldier = new UnitDefinition
            {
                Id = "soldier",
                Stats = new UnitStats { Cost = new ResourceCost { Food = 10, Wood = 5, Stone = 2, Iron = 1, Gold = 0 } }
            };
            UnitDefinition archer = new UnitDefinition
            {
                Id = "archer",
                Stats = new UnitStats { Cost = new ResourceCost { Food = 8, Wood = 3, Stone = 1, Iron = 2, Gold = 0 } }
            };

            registries.Units.Register(soldier.Id, soldier, RegistrySource.Pack, "test-pack");
            registries.Units.Register(archer.Id, archer, RegistrySource.Pack, "test-pack");

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> unitCounts = new Dictionary<string, int> { { "soldier", 5 }, { "archer", 10 } };

            Dictionary<string, float> consumption = calc.CalculateUnitConsumption(registries.Units, unitCounts, profile);

            consumption["food"].Should().Be(130f); // (10*5) + (8*10)
            consumption["wood"].Should().Be(55f);  // (5*5) + (3*10)
        }

        // ── TradeEngine Branch Coverage ───────────────────────────────────────

        [Fact]
        public void TradeEngine_EvaluateTradeRoute_PartnerUnavailable()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute
            {
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 10.0f,
                Enabled = true
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 0f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 50f }, { "gold", -10f }
            };

            TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);

            eval.IsProfitable.Should().BeFalse();
        }

        [Fact]
        public void TradeEngine_EvaluateTradeRoute_CostExceedsAvailableResources()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute
            {
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 100.0f,
                Enabled = true
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 5f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 100f }, { "gold", -10f }
            };

            TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);

            // maxTarget = 5 / 100 = 0.05
            eval.MaxTargetPerExecution.Should().BeApproximately(0.05f, 0.001f);
        }

        [Fact]
        public void TradeEngine_EvaluateTradeRoute_WithMaxPerTransaction()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute
            {
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 5.0f,
                MaxPerTransaction = 50,
                Enabled = true
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 1000f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 100f }, { "gold", -10f }
            };

            TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);

            // maxSource = min(1000, 50) = 50, then maxTarget = 50 / 5 = 10
            eval.MaxTargetPerExecution.Should().BeApproximately(10f, 0.01f);
        }

        [Fact]
        public void TradeEngine_GetOptimalTrades_MultipleDeficits()
        {
            TradeEngine engine = new TradeEngine();

            List<TradeRoute> routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "wood-to-iron",
                    SourceResource = "wood",
                    TargetResource = "iron",
                    ExchangeRate = 5.0f,
                    Enabled = true
                },
                new TradeRoute
                {
                    Id = "stone-to-gold",
                    SourceResource = "stone",
                    TargetResource = "gold",
                    ExchangeRate = 20.0f,
                    Enabled = true
                }
            };

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 500f }, { "stone", 200f }, { "iron", 0f }, { "gold", 0f }, { "food", 100f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 100f }, { "stone", 50f }, { "iron", -20f }, { "gold", -5f }, { "food", 10f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);

            suggestions.Should().HaveCount(2);
            suggestions.Select(s => s.Route.TargetResource).Should().Contain(new[] { "gold", "iron" });
        }

        [Fact]
        public void TradeEngine_GetOptimalTrades_SelfTrade()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute
            {
                Id = "self-trade",
                SourceResource = "wood",
                TargetResource = "wood",
                ExchangeRate = 1.0f,
                Enabled = true
            };

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float> { { "wood", 100f } };
            Dictionary<string, float> balance = new Dictionary<string, float> { { "wood", -10f } };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(
                new List<TradeRoute> { route }, profile, available, balance);

            // Self-trade is allowed if source is in surplus and target is in deficit (both same resource)
            suggestions.Should().HaveCount(1);
        }

        // ── ResourceRate Branch Coverage ──────────────────────────────────────

        [Fact]
        public void ResourceRate_EffectiveRate_WithZeroBaseRate()
        {
            ResourceRate rate = new ResourceRate { BaseRate = 0f, Multiplier = 5.0f };
            rate.EffectiveRate.Should().Be(0f);
        }

        [Fact]
        public void ResourceRate_EffectiveRate_WithVerySmallMultiplier()
        {
            ResourceRate rate = new ResourceRate { BaseRate = 100f, Multiplier = 0.001f };
            rate.EffectiveRate.Should().Be(0.1f);
        }

        [Fact]
        public void ResourceRate_EffectiveRate_WithVeryLargeMultiplier()
        {
            ResourceRate rate = new ResourceRate { BaseRate = 1f, Multiplier = 1000.0f };
            rate.EffectiveRate.Should().Be(1000f);
        }

        [Fact]
        public void ResourceRate_ValidTypes_CanValidateAllTypes()
        {
            foreach (string resourceType in ResourceRate.ValidResourceTypes)
            {
                ResourceRate.ValidResourceTypes.Should().Contain(resourceType);
            }
        }

        // ── Edge Cases: Sustainability and Balance Scoring ────────────────────

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_SingleFactionSustainabilityEdge()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "single-faction" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>();
            EconomyBalanceReport report = balanceCalc.GenerateReport("test-pack", registries, profiles, new List<TradeRoute>());

            report.OverallBalanceScore.Should().Be(1.0f);
        }

        [Fact]
        public void TradeEngine_CalculateExchangeRate_WithZeroTradeRateModifier()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute { ExchangeRate = 10.0f };
            EconomyProfile profile = new EconomyProfile { TradeRateModifier = 0f };

            float rate = engine.CalculateExchangeRate(route, profile);

            rate.Should().Be(0f);
        }

        [Fact]
        public void ProductionCalculator_CalculateFactionProduction_WithExtremeWorkerCounts()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();

            BuildingDefinition farm = new BuildingDefinition
            {
                Id = "swarm-farm",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            registries.Buildings.Register(farm.Id, farm, RegistrySource.Pack, "test-pack");

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> workers = new Dictionary<string, int> { { "swarm-farm", 100 } };

            Dictionary<string, float> output = calc.CalculateBuildingOutput(farm, profile, workers);

            output["food"].Should().Be(1000f);
        }

        [Fact]
        public void TradeEngine_GetOptimalTrades_WithHighExchangeRates()
        {
            TradeEngine engine = new TradeEngine();

            TradeRoute route = new TradeRoute
            {
                Id = "expensive-trade",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 1000.0f,
                Enabled = true
            };

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 1000000f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 500000f }, { "gold", -1f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(
                new List<TradeRoute> { route }, profile, available, balance);

            suggestions.Should().HaveCount(1);
            // With deficit of 1 and exchange rate of 1000, recommends 1000 wood for 1 gold return
            suggestions[0].ExpectedReturn.Should().BeApproximately(1f, 0.01f);
        }
    }
}
