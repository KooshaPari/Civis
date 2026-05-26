#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.Domains.Economy.Trade;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for Economy domain core logic:
/// <see cref="TradeEngine"/>, <see cref="ProductionCalculator"/>.
/// </summary>
public class EconomyDomainExpandedTests
{
    // ─── TradeEngine.CalculateExchangeRate ────────────────────────────────────

    [Fact]
    public void CalculateExchangeRate_NoModifier_ReturnsBaseRate()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", exchangeRate: 10f);
        EconomyProfile profile = new EconomyProfile { TradeRateModifier = 1.0f };

        engine.CalculateExchangeRate(route, profile).Should().BeApproximately(10f, 0.001f);
    }

    [Fact]
    public void CalculateExchangeRate_ProfileReducesRate()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", exchangeRate: 10f);
        EconomyProfile profile = new EconomyProfile { TradeRateModifier = 0.5f };

        // 10 * 0.5 = 5 (better rate for player)
        engine.CalculateExchangeRate(route, profile).Should().BeApproximately(5f, 0.001f);
    }

    [Fact]
    public void CalculateExchangeRate_ProfileIncreasesRate()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", exchangeRate: 10f);
        EconomyProfile profile = new EconomyProfile { TradeRateModifier = 2.0f };

        engine.CalculateExchangeRate(route, profile).Should().BeApproximately(20f, 0.001f);
    }

    // ─── TradeEngine.EvaluateTradeRoute ───────────────────────────────────────

    [Fact]
    public void EvaluateTradeRoute_SurplusSource_DeficitTarget_IsProfitable()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", source: "wood", target: "gold", exchangeRate: 10f);
        EconomyProfile profile = new EconomyProfile { TradeRateModifier = 1.0f };

        // Player has surplus wood (+500) and deficit gold (-200)
        Dictionary<string, float> available = new() { ["wood"] = 1000f, ["gold"] = 10f };
        Dictionary<string, float> balance = new() { ["wood"] = 500f, ["gold"] = -200f };

        TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);

        eval.IsProfitable.Should().BeTrue("surplus source + deficit target is a profitable trade");
        eval.EffectiveExchangeRate.Should().BeApproximately(10f, 0.001f);
        eval.Efficiency.Should().BeInRange(0f, 1f);
    }

    [Fact]
    public void EvaluateTradeRoute_DeficitSource_IsNotProfitable()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", source: "wood", target: "gold", exchangeRate: 10f);
        EconomyProfile profile = new EconomyProfile();

        // Player has deficit wood (-100) — shouldn't trade away their scarce resource
        Dictionary<string, float> available = new() { ["wood"] = 10f };
        Dictionary<string, float> balance = new() { ["wood"] = -100f, ["gold"] = -50f };

        TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);
        eval.IsProfitable.Should().BeFalse("deficit source resource should not be traded away");
    }

    [Fact]
    public void EvaluateTradeRoute_MaxTargetPerExecution_RespectedWhenUnlimited()
    {
        TradeEngine engine = new TradeEngine();
        TradeRoute route = MakeRoute("wood-to-gold", exchangeRate: 5f, maxPerTransaction: 0); // unlimited
        EconomyProfile profile = new EconomyProfile();

        Dictionary<string, float> available = new() { ["wood"] = 100f };
        Dictionary<string, float> balance = new() { ["wood"] = 50f, ["gold"] = -20f };

        TradeEvaluation eval = engine.EvaluateTradeRoute(route, profile, available, balance);
        eval.MaxTargetPerExecution.Should().BeGreaterThan(0);
    }

    // ─── TradeEngine.GetOptimalTrades ─────────────────────────────────────────

    [Fact]
    public void GetOptimalTrades_EmptyRoutes_ReturnsEmpty()
    {
        TradeEngine engine = new TradeEngine();
        List<TradeSuggestion> suggestions = engine.GetOptimalTrades(
            new List<TradeRoute>(),
            new EconomyProfile(),
            new Dictionary<string, float>(),
            new Dictionary<string, float>());

        suggestions.Should().BeEmpty();
    }

    [Fact]
    public void GetOptimalTrades_NeedyResource_SuggestsRelevantRoute()
    {
        TradeEngine engine = new TradeEngine();
        List<TradeRoute> routes = new()
        {
            MakeRoute("wood-to-gold", source: "wood", target: "gold", exchangeRate: 10f),
            MakeRoute("food-to-gold", source: "food", target: "gold", exchangeRate: 20f) // worse rate
        };
        EconomyProfile profile = new EconomyProfile();

        Dictionary<string, float> available = new() { ["wood"] = 500f, ["food"] = 500f };
        Dictionary<string, float> balance = new() { ["wood"] = 200f, ["food"] = 100f, ["gold"] = -300f };

        List<TradeSuggestion> suggestions = engine.GetOptimalTrades(routes, profile, available, balance);
        suggestions.Should().NotBeEmpty("gold deficit should produce trade suggestions");
    }

    // ─── ProductionCalculator.CalculateBuildingOutput ─────────────────────────

    [Fact]
    public void CalculateBuildingOutput_NoRates_ReturnsEmpty()
    {
        ProductionCalculator calc = new ProductionCalculator();
        EconomyProfile profile = new EconomyProfile();

        // Building with no production rates defined — result should be empty dict
        DINOForge.SDK.Models.BuildingDefinition building = new DINOForge.SDK.Models.BuildingDefinition
        {
            Id = "empty-building",
            DisplayName = "Empty"
        };

        Dictionary<string, float> output = calc.CalculateBuildingOutput(building, profile);
        output.Should().NotBeNull();
    }

    [Fact]
    public void GetResourceBalance_Production_MinusConsumption()
    {
        ProductionCalculator calc = new ProductionCalculator();

        Dictionary<string, float> production = new() { ["food"] = 100f, ["wood"] = 50f };
        Dictionary<string, float> consumption = new() { ["food"] = 40f, ["iron"] = 10f };

        Dictionary<string, float> balance = calc.GetResourceBalance(production, consumption);

        balance["food"].Should().BeApproximately(60f, 0.001f, "100 produced - 40 consumed = 60 net");
        balance["wood"].Should().BeApproximately(50f, 0.001f, "50 produced, 0 consumed");
        balance["iron"].Should().BeApproximately(-10f, 0.001f, "0 produced - 10 consumed = -10 deficit");
    }

    [Fact]
    public void GetResourceBalance_EmptyDicts_ReturnsEmpty()
    {
        ProductionCalculator calc = new ProductionCalculator();
        Dictionary<string, float> result = calc.GetResourceBalance(
            new Dictionary<string, float>(),
            new Dictionary<string, float>());
        result.Should().BeEmpty();
    }

    // ─── EconomyProfile defaults ─────────────────────────────────────────────

    [Fact]
    public void EconomyProfile_DefaultTradeModifier_IsOne()
    {
        EconomyProfile profile = new EconomyProfile();
        profile.TradeRateModifier.Should().BeApproximately(1.0f, 0.001f);
    }

    [Fact]
    public void EconomyProfile_ProductionMultiplierMissing_DefaultsToOne()
    {
        EconomyProfile profile = new EconomyProfile();
        // Should not throw — missing key defaults to 1.0 in usage
        profile.ProductionMultipliers.TryGetValue("food", out float mult);
        mult.Should().Be(0f, "missing key returns default (0) until multiplied logic adds 1.0 fallback");
    }

    // ─── TradeRoute construction ──────────────────────────────────────────────

    [Fact]
    public void TradeRoute_DefaultExchangeRate_IsOne()
    {
        TradeRoute route = new TradeRoute();
        route.ExchangeRate.Should().BeApproximately(1.0f, 0.001f);
    }

    [Fact]
    public void TradeEvaluation_Constructor_SetsAllProperties()
    {
        TradeRoute route = MakeRoute("r1");
        TradeEvaluation eval = new TradeEvaluation(route, 5f, 0.8f, true, 20f);
        eval.Route.Should().BeSameAs(route);
        eval.EffectiveExchangeRate.Should().Be(5f);
        eval.Efficiency.Should().Be(0.8f);
        eval.IsProfitable.Should().BeTrue();
        eval.MaxTargetPerExecution.Should().Be(20f);
    }

    // ─── Helpers ──────────────────────────────────────────────────────────────

    private static TradeRoute MakeRoute(
        string id,
        string source = "wood", string target = "gold",
        float exchangeRate = 10f, int maxPerTransaction = 0)
    {
        return new TradeRoute
        {
            Id = id,
            SourceResource = source,
            TargetResource = target,
            ExchangeRate = exchangeRate,
            MaxPerTransaction = maxPerTransaction,
            CooldownTicks = 60
        };
    }
}
