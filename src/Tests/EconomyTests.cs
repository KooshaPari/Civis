using System;
using System.Collections.Generic;
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
    public class EconomyTests
    {
        // ── ResourceRate ────────────────────────────────────

        [Fact]
        public void ResourceRate_EffectiveRate_MultipliesBaseByMultiplier()
        {
            ResourceRate rate = new ResourceRate { BaseRate = 10f, Multiplier = 1.5f };
            rate.EffectiveRate.Should().Be(15f);
        }

        [Fact]
        public void ResourceRate_DefaultMultiplier_IsOne()
        {
            ResourceRate rate = new ResourceRate { BaseRate = 5f };
            rate.EffectiveRate.Should().Be(5f);
        }

        [Fact]
        public void ResourceRate_ValidTypes_ContainsAllFiveResources()
        {
            ResourceRate.ValidResourceTypes.Should().BeEquivalentTo(
                new[] { "food", "wood", "stone", "iron", "gold" });
        }

        // ── EconomyProfile ──────────────────────────────────

        [Fact]
        public void EconomyProfile_GetProductionMultiplier_ReturnsSetValue()
        {
            EconomyProfile profile = new EconomyProfile();
            profile.ProductionMultipliers["wood"] = 2.0f;
            profile.GetProductionMultiplier("wood").Should().Be(2.0f);
        }

        [Fact]
        public void EconomyProfile_GetProductionMultiplier_DefaultsToOne()
        {
            EconomyProfile profile = new EconomyProfile();
            profile.GetProductionMultiplier("stone").Should().Be(1.0f);
        }

        [Fact]
        public void EconomyProfile_GetConsumptionMultiplier_DefaultsToOne()
        {
            EconomyProfile profile = new EconomyProfile();
            profile.GetConsumptionMultiplier("food").Should().Be(1.0f);
        }

        // ── TradeRoute ──────────────────────────────────────

        [Fact]
        public void TradeRoute_DefaultValues_AreValid()
        {
            TradeRoute route = new TradeRoute();
            route.ExchangeRate.Should().Be(1.0f);
            route.CooldownTicks.Should().Be(60);
            route.Enabled.Should().BeTrue();
        }

        // ── TradeEngine ─────────────────────────────────────

        [Fact]
        public void TradeEngine_CalculateExchangeRate_AppliesProfileModifier()
        {
            TradeEngine engine = new TradeEngine();
            TradeRoute route = new TradeRoute { ExchangeRate = 10.0f };
            EconomyProfile profile = new EconomyProfile { TradeRateModifier = 0.8f };

            float rate = engine.CalculateExchangeRate(route, profile);
            rate.Should().Be(8.0f);
        }

        [Fact]
        public void TradeEngine_EvaluateTradeRoute_DetectsProfitableTradeWhenSurplusAndDeficit()
        {
            TradeEngine engine = new TradeEngine();
            TradeRoute route = new TradeRoute
            {
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 10.0f,
                Enabled = true
            };
            EconomyProfile profile = new EconomyProfile { TradeRateModifier = 1.0f };
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 500f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 50f }, { "gold", -10f }
            };

            TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);
            eval.IsProfitable.Should().BeTrue();
            eval.EffectiveExchangeRate.Should().Be(10.0f);
            eval.MaxTargetPerExecution.Should().Be(50f); // 500 / 10
        }

        [Fact]
        public void TradeEngine_EvaluateTradeRoute_NotProfitableWhenNoSurplus()
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
                { "wood", 100f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", -5f }, { "gold", -10f }
            };

            TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);
            eval.IsProfitable.Should().BeFalse();
        }

        [Fact]
        public void TradeEngine_GetOptimalTrades_SuggestsTradeForDeficit()
        {
            TradeEngine engine = new TradeEngine();
            List<TradeRoute> routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "wood-to-gold",
                    SourceResource = "wood",
                    TargetResource = "gold",
                    ExchangeRate = 10.0f,
                    Enabled = true
                }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 1000f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 50f }, { "gold", -20f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);
            suggestions.Should().HaveCount(1);
            suggestions[0].Route.Id.Should().Be("wood-to-gold");
            suggestions[0].ExpectedReturn.Should().BeGreaterThan(0);
        }

        [Fact]
        public void TradeEngine_GetOptimalTrades_ReturnsEmptyWhenNoDeficits()
        {
            TradeEngine engine = new TradeEngine();
            List<TradeRoute> routes = new List<TradeRoute>
            {
                new TradeRoute { SourceResource = "wood", TargetResource = "gold", ExchangeRate = 10.0f, Enabled = true }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 1000f }, { "gold", 500f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 50f }, { "gold", 10f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);
            suggestions.Should().BeEmpty();
        }

        // ── ProductionCalculator ────────────────────────────

        [Fact]
        public void ProductionCalculator_BuildingOutput_AppliesProfileMultiplier()
        {
            ProductionCalculator calc = new ProductionCalculator();
            BuildingDefinition farm = new BuildingDefinition
            {
                Id = "farm",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            EconomyProfile profile = new EconomyProfile();
            profile.ProductionMultipliers["food"] = 1.5f;

            Dictionary<string, float> output = calc.CalculateBuildingOutput(farm, profile);
            output["food"].Should().Be(15f); // 10 * 1.5 * 1(worker) * 1.0(efficiency)
        }

        [Fact]
        public void ProductionCalculator_BuildingOutput_AppliesWorkerCount()
        {
            ProductionCalculator calc = new ProductionCalculator();
            BuildingDefinition mine = new BuildingDefinition
            {
                Id = "mine",
                Production = new Dictionary<string, int> { { "iron", 5 } }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> workers = new Dictionary<string, int> { { "mine", 3 } };

            Dictionary<string, float> output = calc.CalculateBuildingOutput(mine, profile, workers);
            output["iron"].Should().Be(15f); // 5 * 1.0 * 3 * 1.0
        }

        [Fact]
        public void ProductionCalculator_ResourceBalance_CalculatesNetCorrectly()
        {
            ProductionCalculator calc = new ProductionCalculator();
            Dictionary<string, float> production = new Dictionary<string, float>
            {
                { "food", 100f }, { "wood", 50f }
            };
            Dictionary<string, float> consumption = new Dictionary<string, float>
            {
                { "food", 80f }, { "iron", 10f }
            };

            Dictionary<string, float> balance = calc.GetResourceBalance(production, consumption);
            balance["food"].Should().Be(20f);
            balance["wood"].Should().Be(50f);
            balance["iron"].Should().Be(-10f);
        }

        // ── EconomyPlugin ───────────────────────────────────

        [Fact]
        public void EconomyPlugin_Constructor_InitializesAllSubsystems()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            plugin.Production.Should().NotBeNull();
            plugin.Trade.Should().NotBeNull();
            plugin.Balance.Should().NotBeNull();
            plugin.Validator.Should().NotBeNull();
        }

        [Fact]
        public void EconomyPlugin_ValidatePack_ReturnsValidForEmptyPack()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            EconomyValidationResult result = plugin.ValidatePack(
                "test-pack",
                new List<EconomyProfile>(),
                new List<TradeRoute>());

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void EconomyPlugin_ValidatePack_RejectsEmptyPackId()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            System.Action act = () => plugin.ValidatePack(
                "",
                new List<EconomyProfile>(),
                new List<TradeRoute>());

            act.Should().Throw<System.ArgumentException>();
        }

        // ── ProductionCalculator.CalculateFactionProduction ──

        [Fact]
        public void ProductionCalculator_CalculateFactionProduction_AggregatesOutputFromMultipleBuildings()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();

            BuildingDefinition farm = new BuildingDefinition
            {
                Id = "farm",
                Production = new Dictionary<string, int> { { "food", 10 } }
            };
            BuildingDefinition mill = new BuildingDefinition
            {
                Id = "mill",
                Production = new Dictionary<string, int> { { "food", 5 } }
            };

            registries.Buildings.Register(farm.Id, farm, RegistrySource.Pack, "test-pack");
            registries.Buildings.Register(mill.Id, mill, RegistrySource.Pack, "test-pack");

            EconomyProfile profile = new EconomyProfile();
            List<string> buildingIds = new List<string> { "farm", "mill" };

            Dictionary<string, float> production = calc.CalculateFactionProduction(
                "faction1", profile, registries.Buildings, buildingIds);

            production["food"].Should().Be(15f); // 10 + 5
        }

        [Fact]
        public void ProductionCalculator_CalculateFactionProduction_EmptyBuildings_ReturnsZero()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();
            EconomyProfile profile = new EconomyProfile();
            List<string> buildingIds = new List<string>();

            Dictionary<string, float> production = calc.CalculateFactionProduction(
                "faction1", profile, registries.Buildings, buildingIds);

            production.Should().HaveCount(5);
            foreach (string resourceType in ResourceRate.ValidResourceTypes)
            {
                production[resourceType].Should().Be(0f);
            }
        }

        // ── ProductionCalculator.CalculateUnitConsumption ──

        [Fact]
        public void ProductionCalculator_CalculateUnitConsumption_CalculatesUpkeepCorrectly()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();

            UnitDefinition unit = new UnitDefinition
            {
                Id = "soldier",
                Stats = new UnitStats { Cost = new ResourceCost { Food = 5, Wood = 2, Stone = 1, Iron = 0, Gold = 0 } }
            };

            registries.Units.Register(unit.Id, unit, RegistrySource.Pack, "test-pack");

            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> unitCounts = new Dictionary<string, int> { { "soldier", 10 } };

            Dictionary<string, float> consumption = calc.CalculateUnitConsumption(
                registries.Units, unitCounts, profile);

            consumption["food"].Should().Be(50f); // 5 * 10
            consumption["wood"].Should().Be(20f); // 2 * 10
            consumption["stone"].Should().Be(10f); // 1 * 10
        }

        [Fact]
        public void ProductionCalculator_CalculateUnitConsumption_EmptyUnits_ReturnsEmpty()
        {
            ProductionCalculator calc = new ProductionCalculator();
            RegistryManager registries = new RegistryManager();
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, int> unitCounts = new Dictionary<string, int>();

            Dictionary<string, float> consumption = calc.CalculateUnitConsumption(
                registries.Units, unitCounts, profile);

            consumption.Should().BeEmpty();
        }

        // ── EconomyBalanceCalculator.GenerateReport ──

        [Fact]
        public void EconomyBalanceCalculator_GenerateReport_ReturnsNonNullReportWithPackId()
        {
            RegistryManager registries = new RegistryManager();

            FactionDefinition faction = new FactionDefinition
            {
                Faction = new FactionInfo { Id = "knights" },
                Buildings = new FactionBuildings(),
                Economy = new FactionEconomy()
            };
            registries.Factions.Register(faction.Faction.Id, faction, RegistrySource.Pack, "test-pack");

            ProductionCalculator prodCalc = new ProductionCalculator();
            TradeEngine tradeEngine = new TradeEngine();
            EconomyBalanceCalculator balanceCalc = new EconomyBalanceCalculator(prodCalc, tradeEngine);

            Dictionary<string, EconomyProfile> profiles = new Dictionary<string, EconomyProfile>();
            List<TradeRoute> routes = new List<TradeRoute>();

            EconomyBalanceReport report = balanceCalc.GenerateReport(
                "test-pack", registries, profiles, routes);

            report.Should().NotBeNull();
            report.PackId.Should().Be("test-pack");
            report.FactionSummaries.Should().NotBeNull();
            report.OverallBalanceScore.Should().BeGreaterThanOrEqualTo(0f);
            report.Warnings.Should().NotBeNull();
        }

        // ── EconomyValidator.Validate_ValidPack ──

        [Fact]
        public void EconomyValidator_Validate_ValidPack_PassesValidation()
        {
            RegistryManager registries = new RegistryManager();

            EconomyProfile profile = new EconomyProfile
            {
                Id = "economy1",
                DisplayName = "Standard Economy",
                StartingResources = new ResourceCost { Food = 100, Wood = 50, Stone = 50, Iron = 20, Gold = 0 }
            };

            TradeRoute route = new TradeRoute
            {
                Id = "wood-to-gold",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 10.0f,
                Enabled = true
            };

            EconomyValidator validator = new EconomyValidator();
            EconomyValidationResult result = validator.Validate(
                "test-pack",
                registries,
                new List<EconomyProfile> { profile },
                new List<TradeRoute> { route });

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        // ── EconomyValidator.Validate_EmptyPackId ──

        [Fact]
        public void EconomyValidator_Validate_EmptyPackId_Throws()
        {
            RegistryManager registries = new RegistryManager();
            EconomyValidator validator = new EconomyValidator();

            System.Action act = () => validator.Validate(
                "",
                registries,
                new List<EconomyProfile>(),
                new List<TradeRoute>());

            act.Should().Throw<System.ArgumentException>();
        }

        // ── EconomyValidator.Validate_NoProfiles ──

        [Fact]
        public void EconomyValidator_Validate_NoProfiles_ReturnsValid()
        {
            RegistryManager registries = new RegistryManager();
            EconomyValidator validator = new EconomyValidator();

            EconomyValidationResult result = validator.Validate(
                "test-pack",
                registries,
                new List<EconomyProfile>(),
                new List<TradeRoute>());

            // No profiles is valid; validator just doesn't find any to check
            result.IsValid.Should().BeTrue();
        }

        // ── TradeEngine.GetOptimalTrades with disabled route ──

        [Fact]
        public void TradeEngine_GetOptimalTrades_DisabledRoute_IsNotSuggested()
        {
            TradeEngine engine = new TradeEngine();
            List<TradeRoute> routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "disabled-trade",
                    SourceResource = "wood",
                    TargetResource = "gold",
                    ExchangeRate = 10.0f,
                    Enabled = false // Disabled
                }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 1000f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 50f }, { "gold", -20f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);

            suggestions.Should().BeEmpty();
        }

        // ── TradeEngine.GetOptimalTrades with zero resources ──

        [Fact]
        public void TradeEngine_GetOptimalTrades_ZeroAvailableResources_ReturnsNoSuggestions()
        {
            TradeEngine engine = new TradeEngine();
            List<TradeRoute> routes = new List<TradeRoute>
            {
                new TradeRoute
                {
                    Id = "wood-to-gold",
                    SourceResource = "wood",
                    TargetResource = "gold",
                    ExchangeRate = 10.0f,
                    Enabled = true
                }
            };
            EconomyProfile profile = new EconomyProfile();
            Dictionary<string, float> available = new Dictionary<string, float>
            {
                { "wood", 0f }, { "gold", 0f }
            };
            Dictionary<string, float> balance = new Dictionary<string, float>
            {
                { "wood", 0f }, { "gold", -20f }
            };

            List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);

            suggestions.Should().BeEmpty();
        }

        // ── FactionEconomySummary ────────────────────────────────────────────

        [Fact]
        public void FactionEconomySummary_ConstructorSetsAllProperties()
        {
            var production = new Dictionary<string, float> { ["gold"] = 10f, ["wood"] = 5f };
            var consumption = new Dictionary<string, float> { ["gold"] = 3f };
            var netBalance = new Dictionary<string, float> { ["gold"] = 7f, ["wood"] = 5f };

            FactionEconomySummary summary = new FactionEconomySummary(
                factionId: "test-faction",
                production: production,
                consumption: consumption,
                netBalance: netBalance,
                tradeEfficiency: 0.8f,
                sustainabilityScore: 0.9f,
                deficitCount: 0,
                surplusCount: 2);

            summary.FactionId.Should().Be("test-faction");
            summary.Production.Should().ContainKey("gold");
            summary.Consumption.Should().ContainKey("gold");
            summary.NetBalance["gold"].Should().Be(7f);
            summary.TradeEfficiency.Should().Be(0.8f);
            summary.SustainabilityScore.Should().Be(0.9f);
            summary.DeficitCount.Should().Be(0);
            summary.SurplusCount.Should().Be(2);
        }

        [Fact]
        public void FactionEconomySummary_WithDeficits()
        {
            var production = new Dictionary<string, float> { ["gold"] = 1f };
            var consumption = new Dictionary<string, float> { ["gold"] = 5f, ["wood"] = 3f };
            var netBalance = new Dictionary<string, float> { ["gold"] = -4f, ["wood"] = -3f };

            FactionEconomySummary summary = new FactionEconomySummary(
                factionId: "deficit-faction",
                production: production,
                consumption: consumption,
                netBalance: netBalance,
                tradeEfficiency: 0.3f,
                sustainabilityScore: 0.1f,
                deficitCount: 2,
                surplusCount: 0);

            summary.DeficitCount.Should().Be(2);
            summary.SurplusCount.Should().Be(0);
            summary.SustainabilityScore.Should().Be(0.1f);
        }

        // ── EconomyPlugin missing-branch coverage ────────────────────────────

        [Fact]
        public void EconomyPlugin_GenerateBalanceReport_NullProfiles_Throws()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            Action act = () => plugin.GenerateBalanceReport("test-pack", null!, new List<TradeRoute>());
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void EconomyPlugin_GenerateBalanceReport_NullTradeRoutes_Throws()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            Action act = () => plugin.GenerateBalanceReport("test-pack", new Dictionary<string, EconomyProfile>(), null!);
            act.Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void EconomyPlugin_GenerateBalanceReport_EmptyPackId_Throws()
        {
            RegistryManager registries = new RegistryManager();
            EconomyPlugin plugin = new EconomyPlugin(registries);

            Action act = () => plugin.GenerateBalanceReport("", new Dictionary<string, EconomyProfile>(), new List<TradeRoute>());
            act.Should().Throw<ArgumentException>();
        }
    }
}
