using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.Domains.Economy.Rates;
using DINOForge.SDK.Models;
using DINOForge.SDK.Registry;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for Economy domain edge cases and branch coverage gaps.
    /// Focuses on boundary conditions: zero rates, negative values, overflow handling,
    /// and deficit/surplus detection logic in ProductionCalculator and balance models.
    /// </summary>
    public class EconomyCoverageGapTests
    {
        [Fact]
        public void ResourceRate_ZeroBaseRate_EffectiveRateIsZero()
        {
            // Arrange
            var rate = new ResourceRate { BaseRate = 0f, Multiplier = 2.0f };

            // Act
            float effective = rate.EffectiveRate;

            // Assert
            effective.Should().Be(0f, "Zero base rate should result in zero effective rate");
        }

        [Fact]
        public void ResourceRate_NegativeBaseRate_EffectiveRateIsNegative()
        {
            // Arrange
            var rate = new ResourceRate { BaseRate = -10f, Multiplier = 1.5f };

            // Act
            float effective = rate.EffectiveRate;

            // Assert
            effective.Should().Be(-15f, "Negative base rate should multiply correctly");
        }

        [Fact]
        public void ResourceRate_VeryLargeMultiplier_NoOverflow()
        {
            // Arrange
            var rate = new ResourceRate { BaseRate = 100f, Multiplier = 1_000_000f };

            // Act
            float effective = rate.EffectiveRate;

            // Assert
            effective.Should().Be(100_000_000f, "Should handle very large multipliers");
        }

        [Fact]
        public void ResourceRate_ZeroMultiplier_ResultsInZero()
        {
            // Arrange
            var rate = new ResourceRate { BaseRate = 100f, Multiplier = 0f };

            // Act
            float effective = rate.EffectiveRate;

            // Assert
            effective.Should().Be(0f, "Zero multiplier should result in zero effective rate");
        }

        [Fact]
        public void EconomyProfile_GetProductionMultiplier_DefaultsToOne()
        {
            // Arrange
            var profile = new EconomyProfile();

            // Act
            float multiplier = profile.GetProductionMultiplier("nonexistent-resource");

            // Assert
            multiplier.Should().Be(1.0f, "Missing multiplier should default to 1.0");
        }

        [Fact]
        public void EconomyProfile_GetConsumptionMultiplier_DefaultsToOne()
        {
            // Arrange
            var profile = new EconomyProfile();

            // Act
            float multiplier = profile.GetConsumptionMultiplier("nonexistent-resource");

            // Assert
            multiplier.Should().Be(1.0f, "Missing consumption multiplier should default to 1.0");
        }

        [Fact]
        public void EconomyProfile_ProductionMultipliers_CanBeSet()
        {
            // Arrange
            var profile = new EconomyProfile();
            profile.ProductionMultipliers["wood"] = 2.5f;

            // Act
            float retrieved = profile.GetProductionMultiplier("wood");

            // Assert
            retrieved.Should().Be(2.5f, "Set production multiplier should be retrieved correctly");
        }

        [Fact]
        public void EconomyProfile_ConsumptionMultipliers_CanBeSet()
        {
            // Arrange
            var profile = new EconomyProfile();
            profile.ConsumptionMultipliers["food"] = 0.5f;

            // Act
            float retrieved = profile.GetConsumptionMultiplier("food");

            // Assert
            retrieved.Should().Be(0.5f, "Set consumption multiplier should be retrieved correctly");
        }

        [Fact]
        public void EconomyProfile_TradeRateModifier_DefaultIsOne()
        {
            // Arrange
            var profile = new EconomyProfile();

            // Act & Assert
            profile.TradeRateModifier.Should().Be(1.0f, "Default trade rate modifier should be 1.0");
        }

        [Fact]
        public void EconomyProfile_TradeRateModifier_CanBeNegative()
        {
            // Arrange
            var profile = new EconomyProfile { TradeRateModifier = -0.5f };

            // Act & Assert
            profile.TradeRateModifier.Should().Be(-0.5f, "Negative trade rate modifiers should be allowed");
        }

        [Fact]
        public void EconomyProfile_TradeRateModifier_CanBeVeryLarge()
        {
            // Arrange
            var profile = new EconomyProfile { TradeRateModifier = 100f };

            // Act & Assert
            profile.TradeRateModifier.Should().Be(100f, "Very large trade rate modifiers should be allowed");
        }

        [Fact]
        public void TradeRoute_DefaultValues()
        {
            // Arrange & Act
            var route = new TradeRoute();

            // Assert
            route.ExchangeRate.Should().Be(1.0f, "Default exchange rate should be 1.0");
            route.CooldownTicks.Should().Be(60, "Default cooldown should be 60 ticks");
            route.Enabled.Should().BeTrue("Trade routes should be enabled by default");
        }

        [Fact]
        public void TradeRoute_CanBeDisabled()
        {
            // Arrange
            var route = new TradeRoute { Enabled = false };

            // Act & Assert
            route.Enabled.Should().BeFalse("Disabled trade routes should stay disabled");
        }

        [Fact]
        public void TradeRoute_ExchangeRateCanBeZero()
        {
            // Arrange
            var route = new TradeRoute { ExchangeRate = 0f };

            // Act & Assert
            route.ExchangeRate.Should().Be(0f, "Zero exchange rate should be allowed");
        }

        [Fact]
        public void TradeRoute_ExchangeRateCanBeNegative()
        {
            // Arrange
            var route = new TradeRoute { ExchangeRate = -5.0f };

            // Act & Assert
            route.ExchangeRate.Should().Be(-5.0f, "Negative exchange rate should be allowed");
        }

        [Fact]
        public void TradeRoute_CooldownTicksCanBeZero()
        {
            // Arrange
            var route = new TradeRoute { CooldownTicks = 0 };

            // Act & Assert
            route.CooldownTicks.Should().Be(0, "Zero cooldown should be allowed");
        }

        [Fact]
        public void TradeRoute_CooldownTicksCanBeNegative()
        {
            // Arrange
            var route = new TradeRoute { CooldownTicks = -10 };

            // Act & Assert
            route.CooldownTicks.Should().Be(-10, "Negative cooldown should be allowed (for testing)");
        }

        [Fact]
        public void TradeRoute_SourceResourceCanBeNull()
        {
            // Arrange & Act
            var route = new TradeRoute { SourceResource = null! };

            // Assert
            route.SourceResource.Should().BeNull("SourceResource can be null");
        }

        [Fact]
        public void TradeRoute_TargetResourceCanBeNull()
        {
            // Arrange & Act
            var route = new TradeRoute { TargetResource = null! };

            // Assert
            route.TargetResource.Should().BeNull("TargetResource can be null");
        }


        [Fact]
        public void ProductionCalculator_GetResourceBalance_ZeroProduction()
        {
            // Arrange
            var calculator = new ProductionCalculator();
            var production = new Dictionary<string, float>
            {
                { "food", 0f },
                { "wood", 0f }
            };
            var consumption = new Dictionary<string, float>
            {
                { "food", 5f },
                { "wood", 3f }
            };

            // Act
            var balance = calculator.GetResourceBalance(production, consumption);

            // Assert
            balance["food"].Should().Be(-5f, "Zero production with positive consumption creates deficit");
            balance["wood"].Should().Be(-3f, "Zero production with positive consumption creates deficit");
        }

        [Fact]
        public void ProductionCalculator_GetResourceBalance_NegativeProduction()
        {
            // Arrange
            var calculator = new ProductionCalculator();
            var production = new Dictionary<string, float>
            {
                { "food", -10f },
                { "wood", 5f }
            };
            var consumption = new Dictionary<string, float>
            {
                { "food", 5f },
                { "wood", 3f }
            };

            // Act
            var balance = calculator.GetResourceBalance(production, consumption);

            // Assert
            balance["food"].Should().Be(-15f, "Negative production minus consumption creates larger deficit");
            balance["wood"].Should().Be(2f, "Positive production minus consumption creates surplus");
        }

        [Fact]
        public void ProductionCalculator_GetResourceBalance_VeryLargeValues()
        {
            // Arrange
            var calculator = new ProductionCalculator();
            var production = new Dictionary<string, float>
            {
                { "food", 1_000_000f },
                { "wood", 500_000f }
            };
            var consumption = new Dictionary<string, float>
            {
                { "food", 999_999f },
                { "wood", 500_000f }
            };

            // Act
            var balance = calculator.GetResourceBalance(production, consumption);

            // Assert
            balance["food"].Should().Be(1f, "Large numbers should calculate correctly");
            balance["wood"].Should().Be(0f, "Equal production and consumption should result in zero balance");
        }
    }
}
