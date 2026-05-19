using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="EconomyProfile"/>.
    /// Tests constructor defaults, validation rules, and multiplier edge cases.
    /// </summary>
    public class EconomyProfileUnitTests
    {
        /// <summary>
        /// Test that parameterless constructor initializes with expected defaults.
        /// </summary>
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedDefaults()
        {
            var profile = new EconomyProfile();

            profile.Id.Should().BeEmpty();
            profile.DisplayName.Should().BeEmpty();
            profile.Description.Should().BeNull();
            profile.TradeRateModifier.Should().Be(1.0f);
            profile.TradeCooldownModifier.Should().Be(1.0f);
            profile.StorageMultiplier.Should().Be(1.0f);
            profile.BuildingCostModifier.Should().Be(1.0f);
            profile.WorkerEfficiency.Should().Be(1.0f);
            profile.ProductionMultipliers.Should().NotBeNull().And.BeEmpty();
            profile.ConsumptionMultipliers.Should().NotBeNull().And.BeEmpty();
            profile.StartingResources.Should().NotBeNull();
        }

        /// <summary>
        /// Test that Validate() succeeds with valid EconomyProfile.
        /// </summary>
        [Fact]
        public void Validate_WithValidDefinition_ReturnsSuccess()
        {
            var profile = new EconomyProfile
            {
                Id = "balanced-economy",
                DisplayName = "Balanced Economy",
                TradeRateModifier = 1.0f,
                StorageMultiplier = 1.5f,
                WorkerEfficiency = 1.2f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        /// <summary>
        /// Test that Validate() fails with blank Id.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankId_ReturnsError(string? id)
        {
            var profile = new EconomyProfile { Id = id, DisplayName = "Valid Name" };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
        }

        /// <summary>
        /// Test that Validate() fails with blank DisplayName.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankDisplayName_ReturnsError(string? displayName)
        {
            var profile = new EconomyProfile { Id = "test-economy", DisplayName = displayName };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "display_name");
        }

        /// <summary>
        /// Test that Validate() fails with negative TradeRateModifier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeTradeRateModifier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                TradeRateModifier = -0.5f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "trade_rate_modifier");
        }

        /// <summary>
        /// Test that Validate() fails with negative TradeCooldownModifier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeTradeCooldownModifier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                TradeCooldownModifier = -0.1f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "trade_cooldown_modifier");
        }

        /// <summary>
        /// Test that Validate() fails with negative StorageMultiplier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeStorageMultiplier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                StorageMultiplier = -0.5f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "storage_multiplier");
        }

        /// <summary>
        /// Test that Validate() fails with negative BuildingCostModifier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeBuildingCostModifier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                BuildingCostModifier = -1.0f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "building_cost_modifier");
        }

        /// <summary>
        /// Test that Validate() fails with negative WorkerEfficiency.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeWorkerEfficiency_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                WorkerEfficiency = -0.5f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "worker_efficiency");
        }

        /// <summary>
        /// Test that Validate() fails with negative production multiplier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeProductionMultiplier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy"
            };
            profile.ProductionMultipliers["food"] = -0.5f;

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "production_multipliers.food");
        }

        /// <summary>
        /// Test that Validate() fails with negative consumption multiplier.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeConsumptionMultiplier_ReturnsError()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy"
            };
            profile.ConsumptionMultipliers["wood"] = -0.2f;

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "consumption_multipliers.wood");
        }

        /// <summary>
        /// Test that GetProductionMultiplier returns correct value for existing resource.
        /// </summary>
        [Fact]
        public void GetProductionMultiplier_WithExistingResource_ReturnsSetValue()
        {
            var profile = new EconomyProfile();
            profile.ProductionMultipliers["food"] = 1.5f;

            var multiplier = profile.GetProductionMultiplier("food");

            multiplier.Should().Be(1.5f);
        }

        /// <summary>
        /// Test that GetProductionMultiplier defaults to 1.0 for missing resource.
        /// </summary>
        [Fact]
        public void GetProductionMultiplier_WithMissingResource_ReturnsDefault()
        {
            var profile = new EconomyProfile();

            var multiplier = profile.GetProductionMultiplier("nonexistent");

            multiplier.Should().Be(1.0f);
        }

        /// <summary>
        /// Test that GetConsumptionMultiplier returns correct value for existing resource.
        /// </summary>
        [Fact]
        public void GetConsumptionMultiplier_WithExistingResource_ReturnsSetValue()
        {
            var profile = new EconomyProfile();
            profile.ConsumptionMultipliers["stone"] = 0.8f;

            var multiplier = profile.GetConsumptionMultiplier("stone");

            multiplier.Should().Be(0.8f);
        }

        /// <summary>
        /// Test that GetConsumptionMultiplier defaults to 1.0 for missing resource.
        /// </summary>
        [Fact]
        public void GetConsumptionMultiplier_WithMissingResource_ReturnsDefault()
        {
            var profile = new EconomyProfile();

            var multiplier = profile.GetConsumptionMultiplier("nonexistent");

            multiplier.Should().Be(1.0f);
        }

        /// <summary>
        /// Test that Validate() accepts zero as valid modifier value.
        /// </summary>
        [Fact]
        public void Validate_WithZeroModifier_ReturnsSuccess()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                TradeRateModifier = 0.0f,
                StorageMultiplier = 0.0f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() detects multiple modifier errors.
        /// </summary>
        [Fact]
        public void Validate_WithMultipleNegativeModifiers_ReturnsAllErrors()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy",
                TradeRateModifier = -0.5f,
                StorageMultiplier = -1.0f,
                BuildingCostModifier = -0.2f
            };

            var result = profile.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3);
            result.Errors.Should().Contain(e => e.Path == "trade_rate_modifier");
            result.Errors.Should().Contain(e => e.Path == "storage_multiplier");
            result.Errors.Should().Contain(e => e.Path == "building_cost_modifier");
        }

        /// <summary>
        /// Test that multiple resource multipliers can be added and validated.
        /// </summary>
        [Fact]
        public void ProductionMultipliers_CanHoldMultipleResources()
        {
            var profile = new EconomyProfile
            {
                Id = "test-economy",
                DisplayName = "Test Economy"
            };
            profile.ProductionMultipliers["food"] = 1.2f;
            profile.ProductionMultipliers["wood"] = 0.9f;
            profile.ProductionMultipliers["stone"] = 1.5f;

            profile.ProductionMultipliers.Should().HaveCount(3);
            profile.ProductionMultipliers["food"].Should().Be(1.2f);
            profile.ProductionMultipliers["wood"].Should().Be(0.9f);
            profile.ProductionMultipliers["stone"].Should().Be(1.5f);
        }
    }
}
