#nullable enable
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for the Economy <c>ResourceRate</c> (EffectiveRate math + enum Validate) and
/// <c>TradeRouteDefinition</c> (required fields + positive ExchangeRate).
/// </summary>
public class EconomyModelsCoverageTests
{
    // --- ResourceRate ---

    [Fact]
    public void ResourceRate_EffectiveRate_IsBaseTimesMultiplier()
    {
        ResourceRate rate = new() { BaseRate = 4f, Multiplier = 2.5f };

        rate.EffectiveRate.Should().Be(10f);
    }

    [Fact]
    public void ResourceRate_DefaultMultiplier_IsOne()
    {
        ResourceRate rate = new() { BaseRate = 7f };

        rate.Multiplier.Should().Be(1.0f);
        rate.EffectiveRate.Should().Be(7f);
    }

    [Fact]
    public void ResourceRate_DefinedResourceType_IsValid()
    {
        ResourceRate rate = new() { ResourceType = ResourceKind.Food, BaseRate = 1f };

        rate.Validate().IsValid.Should().BeTrue();
    }

    [Fact]
    public void ResourceRate_UndefinedResourceType_FailsWithEnumError()
    {
        ResourceRate rate = new() { ResourceType = (ResourceKind)999, BaseRate = 1f };

        ValidationResult result = rate.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "resource_type");
    }

    // --- TradeRouteDefinition ---

    private static TradeRouteDefinition ValidRoute() => new()
    {
        Id = "route:gold-iron",
        SourceResource = "gold",
        TargetResource = "iron",
        ExchangeRate = 2f
    };

    [Fact]
    public void TradeRoute_FullyPopulated_IsValid()
    {
        ValidRoute().Validate().IsValid.Should().BeTrue();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void TradeRoute_MissingId_Fails(string id)
    {
        TradeRouteDefinition r = ValidRoute();
        r.Id = id;

        r.Validate().Errors.Should().Contain(e => e.Path == "id");
    }

    [Fact]
    public void TradeRoute_MissingSourceResource_Fails()
    {
        TradeRouteDefinition r = ValidRoute();
        r.SourceResource = "";

        r.Validate().Errors.Should().Contain(e => e.Path == "source_resource");
    }

    [Fact]
    public void TradeRoute_MissingTargetResource_Fails()
    {
        TradeRouteDefinition r = ValidRoute();
        r.TargetResource = "";

        r.Validate().Errors.Should().Contain(e => e.Path == "target_resource");
    }

    [Theory]
    [InlineData(0f)]
    [InlineData(-1f)]
    public void TradeRoute_NonPositiveExchangeRate_Fails(float rate)
    {
        TradeRouteDefinition r = ValidRoute();
        r.ExchangeRate = rate;

        ValidationResult result = r.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "exchange_rate");
    }

    [Fact]
    public void TradeRoute_AllInvalid_ReportsMultipleErrors()
    {
        TradeRouteDefinition r = new() { Id = "", SourceResource = "", TargetResource = "", ExchangeRate = 0f };

        r.Validate().Errors.Should().HaveCountGreaterThanOrEqualTo(4);
    }
}
