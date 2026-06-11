#nullable enable
using System;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Trade;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Domains;

/// <summary>
/// Coverage for <see cref="TradeEngine.CalculateExchangeRate"/> (rate * profile-modifier
/// with null guards) and <see cref="EconomyProfile.Validate"/> (required Id/DisplayName +
/// non-negative modifier rules).
/// </summary>
public class TradeEngineCoverageTests
{
    private static EconomyProfile ValidProfile() => new()
    {
        Id = "econ:standard",
        DisplayName = "Standard",
        TradeRateModifier = 1.0f
    };

    // --- TradeEngine.CalculateExchangeRate ---

    [Fact]
    public void CalculateExchangeRate_AppliesProfileModifier()
    {
        TradeRoute route = new() { ExchangeRate = 2.0f };
        EconomyProfile profile = ValidProfile();
        profile.TradeRateModifier = 1.5f;

        float rate = new TradeEngine().CalculateExchangeRate(route, profile);

        rate.Should().Be(3.0f); // 2.0 * 1.5
    }

    [Fact]
    public void CalculateExchangeRate_DefaultModifier_IsIdentity()
    {
        TradeRoute route = new() { ExchangeRate = 4.0f };

        float rate = new TradeEngine().CalculateExchangeRate(route, ValidProfile());

        rate.Should().Be(4.0f); // 4.0 * 1.0 (default modifier)
    }

    [Fact]
    public void CalculateExchangeRate_NullRoute_Throws()
    {
        Action act = () => new TradeEngine().CalculateExchangeRate(null!, ValidProfile());

        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void CalculateExchangeRate_NullProfile_Throws()
    {
        Action act = () => new TradeEngine().CalculateExchangeRate(new TradeRoute(), null!);

        act.Should().Throw<ArgumentNullException>();
    }

    // --- EconomyProfile.Validate ---

    [Fact]
    public void EconomyProfile_Valid_IsValid()
    {
        ValidProfile().Validate().IsValid.Should().BeTrue();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void EconomyProfile_MissingId_Fails(string id)
    {
        EconomyProfile p = ValidProfile();
        p.Id = id;

        p.Validate().Errors.Should().Contain(e => e.Path == "id");
    }

    [Fact]
    public void EconomyProfile_MissingDisplayName_Fails()
    {
        EconomyProfile p = ValidProfile();
        p.DisplayName = "";

        p.Validate().Errors.Should().Contain(e => e.Path == "display_name");
    }

    [Fact]
    public void EconomyProfile_NegativeTradeRateModifier_Fails()
    {
        EconomyProfile p = ValidProfile();
        p.TradeRateModifier = -1f;

        ValidationResult result = p.Validate();

        result.IsValid.Should().BeFalse();
        result.Errors.Should().Contain(e => e.Path == "trade_rate_modifier");
    }

    [Fact]
    public void EconomyProfile_NegativeStorageMultiplier_Fails()
    {
        EconomyProfile p = ValidProfile();
        p.StorageMultiplier = -0.5f;

        p.Validate().Errors.Should().Contain(e => e.Path == "storage_multiplier");
    }
}
