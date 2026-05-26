using System;
using FluentAssertions;
using DINOForge.Domains.Economy.Models;
using Xunit;

namespace DINOForge.Tests
{
    public class TradeRouteDefinitionUnitTests
    {
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedValues()
        {
            var route = new TradeRouteDefinition();

            route.Id.Should().Be(string.Empty);
            route.DisplayName.Should().Be(string.Empty);
            route.SourceResource.Should().Be(string.Empty);
            route.TargetResource.Should().Be(string.Empty);
            route.ExchangeRate.Should().Be(1.0f);
            route.CooldownTicks.Should().Be(60);
            route.MaxPerTransaction.Should().Be(1000.0f);
            route.Enabled.Should().BeTrue();
        }

        [Fact]
        public void FullConstructor_WithValidArguments_InitializesCorrectly()
        {
            var route = new TradeRouteDefinition(
                id: "trade-wood-to-gold",
                displayName: "Wood → Gold",
                sourceResource: "wood",
                targetResource: "gold",
                exchangeRate: 10.0f,
                cooldownTicks: 120,
                maxPerTransaction: 500.0f,
                enabled: true);

            route.Id.Should().Be("trade-wood-to-gold");
            route.DisplayName.Should().Be("Wood → Gold");
            route.SourceResource.Should().Be("wood");
            route.TargetResource.Should().Be("gold");
            route.ExchangeRate.Should().Be(10.0f);
            route.CooldownTicks.Should().Be(120);
            route.MaxPerTransaction.Should().Be(500.0f);
            route.Enabled.Should().BeTrue();
        }

        [Fact]
        public void FullConstructor_WithNullId_ThrowsArgumentNullException()
        {
            var action = () => new TradeRouteDefinition(
                id: null!,
                displayName: "Test",
                sourceResource: "wood",
                targetResource: "gold",
                exchangeRate: 1.0f,
                cooldownTicks: 60,
                maxPerTransaction: 1000.0f,
                enabled: true);

            action.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        [Fact]
        public void FullConstructor_WithNullDisplayName_ThrowsArgumentNullException()
        {
            var action = () => new TradeRouteDefinition(
                id: "trade-001",
                displayName: null!,
                sourceResource: "wood",
                targetResource: "gold",
                exchangeRate: 1.0f,
                cooldownTicks: 60,
                maxPerTransaction: 1000.0f,
                enabled: true);

            action.Should().Throw<ArgumentNullException>().WithParameterName("displayName");
        }

        [Fact]
        public void FullConstructor_WithNullSourceResource_ThrowsArgumentNullException()
        {
            var action = () => new TradeRouteDefinition(
                id: "trade-001",
                displayName: "Test",
                sourceResource: null!,
                targetResource: "gold",
                exchangeRate: 1.0f,
                cooldownTicks: 60,
                maxPerTransaction: 1000.0f,
                enabled: true);

            action.Should().Throw<ArgumentNullException>().WithParameterName("sourceResource");
        }

        [Fact]
        public void FullConstructor_WithNullTargetResource_ThrowsArgumentNullException()
        {
            var action = () => new TradeRouteDefinition(
                id: "trade-001",
                displayName: "Test",
                sourceResource: "wood",
                targetResource: null!,
                exchangeRate: 1.0f,
                cooldownTicks: 60,
                maxPerTransaction: 1000.0f,
                enabled: true);

            action.Should().Throw<ArgumentNullException>().WithParameterName("targetResource");
        }

        [Fact]
        public void Validate_WithValidRoute_ReturnsSuccess()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Wood to Gold",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 5.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithBlankId_ReturnsFailure()
        {
            var route = new TradeRouteDefinition
            {
                Id = "   ",
                DisplayName = "Test",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 1.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        [Fact]
        public void Validate_WithBlankSourceResource_ReturnsFailure()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "",
                TargetResource = "gold",
                ExchangeRate = 1.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "source_resource");
        }

        [Fact]
        public void Validate_WithBlankTargetResource_ReturnsFailure()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "wood",
                TargetResource = "   ",
                ExchangeRate = 1.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "target_resource");
        }

        [Fact]
        public void Validate_WithZeroExchangeRate_ReturnsFailure()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 0.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "exchange_rate");
        }

        [Fact]
        public void Validate_WithNegativeExchangeRate_ReturnsFailure()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = -5.0f
            };

            var result = route.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "exchange_rate");
        }

        [Fact]
        public void Validate_WithPositiveExchangeRate_Succeeds()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "wood",
                TargetResource = "gold",
                ExchangeRate = 0.001f
            };

            var result = route.Validate();

            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void ExchangeRate_CanBeVerySmall()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "diamond",
                TargetResource = "gold",
                ExchangeRate = 0.0001f
            };

            route.ExchangeRate.Should().Be(0.0001f);
        }

        [Fact]
        public void ExchangeRate_CanBeVeryLarge()
        {
            var route = new TradeRouteDefinition
            {
                Id = "trade-001",
                DisplayName = "Test",
                SourceResource = "sand",
                TargetResource = "gold",
                ExchangeRate = 10000.0f
            };

            route.ExchangeRate.Should().Be(10000.0f);
        }

        [Fact]
        public void CooldownTicks_CanBeZero()
        {
            var route = new TradeRouteDefinition { CooldownTicks = 0 };

            route.CooldownTicks.Should().Be(0);
        }

        [Fact]
        public void MaxPerTransaction_CanBeZero()
        {
            var route = new TradeRouteDefinition { MaxPerTransaction = 0.0f };

            route.MaxPerTransaction.Should().Be(0.0f);
        }

        [Fact]
        public void Enabled_DefaultsToTrue()
        {
            var route = new TradeRouteDefinition();

            route.Enabled.Should().BeTrue();
        }

        [Fact]
        public void Enabled_CanBeSetToFalse()
        {
            var route = new TradeRouteDefinition { Enabled = false };

            route.Enabled.Should().BeFalse();
        }
    }
}
