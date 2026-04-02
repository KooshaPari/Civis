#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Trade;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Coverage tests for TradeEngine - improves branch coverage in Economy domain.
/// </summary>
public class TradeEngineCoverageTests
{
    private readonly TradeEngine _engine = new();

    // ──────────────────────── CalculateExchangeRate ────────────────────────

    [Fact]
    public void CalculateExchangeRate_WithDefaultModifier_ReturnsRouteRate()
    {
        var route = CreateRoute("food", "gold", 2.0f);
        var profile = CreateProfile(tradeRateModifier: 1.0f);

        float rate = _engine.CalculateExchangeRate(route, profile);

        rate.Should().Be(2.0f);
    }

    [Fact]
    public void CalculateExchangeRate_WithModifier_AppliesModifier()
    {
        var route = CreateRoute("food", "gold", 2.0f);
        var profile = CreateProfile(tradeRateModifier: 0.5f);

        float rate = _engine.CalculateExchangeRate(route, profile);

        rate.Should().Be(1.0f);
    }

    [Fact]
    public void CalculateExchangeRate_WithNullRoute_ThrowsArgumentNullException()
    {
        var profile = CreateProfile();

        Action action = () => _engine.CalculateExchangeRate(null!, profile);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void CalculateExchangeRate_WithNullProfile_ThrowsArgumentNullException()
    {
        var route = CreateRoute();

        Action action = () => _engine.CalculateExchangeRate(route, null!);

        action.Should().Throw<ArgumentNullException>();
    }

    // ──────────────────────── EvaluateTradeRoute ────────────────────────

    [Fact]
    public void EvaluateTradeRoute_WithProfitableTrade_ReturnsProfitableEvaluation()
    {
        var route = CreateRoute("food", "gold", 2.0f, enabled: true);
        var profile = CreateProfile(tradeRateModifier: 1.0f);
        var resources = new Dictionary<string, float> { ["food"] = 100, ["gold"] = 0 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 }; // surplus food, deficit gold

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.IsProfitable.Should().BeTrue();
        eval.Efficiency.Should().BeGreaterThan(0);
        eval.MaxTargetPerExecution.Should().BeGreaterThan(0);
    }

    [Fact]
    public void EvaluateTradeRoute_WithDisabledRoute_ReturnsNotProfitable()
    {
        var route = CreateRoute("food", "gold", 2.0f, enabled: false);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.IsProfitable.Should().BeFalse();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNoSourceSurplus_ReturnsNotProfitable()
    {
        var route = CreateRoute("food", "gold", 2.0f, enabled: true);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 0 }; // no food available
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.IsProfitable.Should().BeFalse();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNoTargetDeficit_ReturnsNotProfitable()
    {
        var route = CreateRoute("food", "gold", 2.0f, enabled: true);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100, ["gold"] = 50 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = 5 }; // no deficit

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.IsProfitable.Should().BeFalse();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNegativeBalance_ReturnsNotProfitable()
    {
        var route = CreateRoute("food", "gold", 2.0f, enabled: true);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = -5, ["gold"] = -5 }; // food deficit

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.IsProfitable.Should().BeFalse();
    }

    [Fact]
    public void EvaluateTradeRoute_WithMaxPerTransaction_CapsOutput()
    {
        var route = CreateRoute("food", "gold", 2.0f, maxPerTransaction: 50);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100, ["gold"] = 0 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.MaxTargetPerExecution.Should().Be(25f); // 50 / 2 = 25, capped by MaxPerTransaction
    }

    [Fact]
    public void EvaluateTradeRoute_WithZeroExchangeRate_ReturnsZeroEfficiency()
    {
        var route = CreateRoute("food", "gold", 0f);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.Efficiency.Should().Be(0f);
        eval.MaxTargetPerExecution.Should().Be(0f);
    }

    [Fact]
    public void EvaluateTradeRoute_WithUnknownResources_HandlesGracefully()
    {
        var route = CreateRoute("wood", "stone", 2.0f);
        var profile = CreateProfile();
        var resources = new Dictionary<string, float>(); // no resources
        var balance = new Dictionary<string, float>(); // no balance

        TradeEvaluation eval = _engine.EvaluateTradeRoute(route, profile, resources, balance);

        eval.Should().NotBeNull();
        eval.IsProfitable.Should().BeFalse();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNullRoute_ThrowsArgumentNullException()
    {
        var profile = CreateProfile();
        var resources = new Dictionary<string, float>();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.EvaluateTradeRoute(null!, profile, resources, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNullProfile_ThrowsArgumentNullException()
    {
        var route = CreateRoute();
        var resources = new Dictionary<string, float>();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.EvaluateTradeRoute(route, null!, resources, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNullResources_ThrowsArgumentNullException()
    {
        var route = CreateRoute();
        var profile = CreateProfile();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.EvaluateTradeRoute(route, profile, null!, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EvaluateTradeRoute_WithNullBalance_ThrowsArgumentNullException()
    {
        var route = CreateRoute();
        var profile = CreateProfile();
        var resources = new Dictionary<string, float>();

        Action action = () => _engine.EvaluateTradeRoute(route, profile, resources, null!);

        action.Should().Throw<ArgumentNullException>();
    }

    // ──────────────────────── GetOptimalTrades ────────────────────────

    [Fact]
    public void GetOptimalTrades_WithNoDeficits_ReturnsEmpty()
    {
        var routes = new List<TradeRoute> { CreateRoute() };
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 5 }; // positive

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().BeEmpty();
    }

    [Fact]
    public void GetOptimalTrades_WithDeficitButNoRoute_ReturnsEmpty()
    {
        var routes = new List<TradeRoute> { CreateRoute("wood", "stone", 2.0f) }; // doesn't produce food
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["wood"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = -10 }; // food deficit

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().BeEmpty();
    }

    [Fact]
    public void GetOptimalTrades_WithDeficitAndRoute_ReturnsSuggestion()
    {
        var routes = new List<TradeRoute> { CreateRoute("food", "gold", 2.0f) };
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100, ["gold"] = 0 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().ContainSingle();
        suggestions[0].RecommendedAmount.Should().BeGreaterThan(0);
        suggestions[0].ExpectedReturn.Should().BeGreaterThan(0);
        suggestions[0].Reason.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void GetOptimalTrades_WithDisabledRoutes_ReturnsEmpty()
    {
        var routes = new List<TradeRoute> { CreateRoute("food", "gold", 2.0f, enabled: false) };
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().BeEmpty();
    }

    [Fact]
    public void GetOptimalTrades_WithRouteProducingZero_ReturnsEmpty()
    {
        var route = CreateRoute("food", "gold", 0f); // zero exchange rate
        var routes = new List<TradeRoute> { route };
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["gold"] = -5 };

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().BeEmpty();
    }

    [Fact]
    public void GetOptimalTrades_WithMultipleRoutes_PicksMostEfficient()
    {
        var routes = new List<TradeRoute>
        {
            CreateRoute("food", "gold", 4.0f), // worse rate
            CreateRoute("wood", "gold", 2.0f), // better rate
        };
        var profile = CreateProfile();
        var resources = new Dictionary<string, float> { ["food"] = 100, ["wood"] = 100 };
        var balance = new Dictionary<string, float> { ["food"] = 10, ["wood"] = 10, ["gold"] = -5 };

        List<TradeSuggestion> suggestions = _engine.GetOptimalTrades(routes, profile, resources, balance);

        suggestions.Should().ContainSingle();
        suggestions[0].Route.TargetResource.Should().Be("gold");
    }

    [Fact]
    public void GetOptimalTrades_WithNullRoutes_ThrowsArgumentNullException()
    {
        var profile = CreateProfile();
        var resources = new Dictionary<string, float>();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.GetOptimalTrades(null!, profile, resources, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void GetOptimalTrades_WithNullProfile_ThrowsArgumentNullException()
    {
        var routes = new List<TradeRoute>();
        var resources = new Dictionary<string, float>();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.GetOptimalTrades(routes, null!, resources, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void GetOptimalTrades_WithNullResources_ThrowsArgumentNullException()
    {
        var routes = new List<TradeRoute>();
        var profile = CreateProfile();
        var balance = new Dictionary<string, float>();

        Action action = () => _engine.GetOptimalTrades(routes, profile, null!, balance);

        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void GetOptimalTrades_WithNullBalance_ThrowsArgumentNullException()
    {
        var routes = new List<TradeRoute>();
        var profile = CreateProfile();
        var resources = new Dictionary<string, float>();

        Action action = () => _engine.GetOptimalTrades(routes, profile, resources, null!);

        action.Should().Throw<ArgumentNullException>();
    }

    // ──────────────────────── Helpers ────────────────────────

    private static TradeRoute CreateRoute(
        string source = "food",
        string target = "gold",
        float exchangeRate = 2.0f,
        bool enabled = true,
        int maxPerTransaction = 0)
    {
        return new TradeRoute
        {
            Id = $"route_{source}_to_{target}",
            SourceResource = source,
            TargetResource = target,
            ExchangeRate = exchangeRate,
            Enabled = enabled,
            MaxPerTransaction = maxPerTransaction,
        };
    }

    private static EconomyProfile CreateProfile(float tradeRateModifier = 1.0f)
    {
        return new EconomyProfile
        {
            Id = "test_profile",
            DisplayName = "Test Profile",
            TradeRateModifier = tradeRateModifier,
            ProductionMultipliers = new Dictionary<string, float>(),
            ConsumptionMultipliers = new Dictionary<string, float>(),
        };
    }
}
