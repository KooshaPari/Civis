using System;
using System.Collections.Generic;
using DINOForge.Domains.Economy.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="ResourceDefinition"/>.
    /// Tests constructor defaults, validation rules, and production/storage edge cases.
    /// </summary>
    public class ResourceDefinitionUnitTests
    {
        /// <summary>
        /// Test that parameterless constructor initializes with expected defaults.
        /// </summary>
        [Fact]
        public void DefaultConstructor_InitializesWithExpectedDefaults()
        {
            var resource = new ResourceDefinition();

            resource.Id.Should().BeEmpty();
            resource.Name.Should().BeEmpty();
            resource.Description.Should().BeEmpty();
            resource.ProductionRate.Should().Be(1.0f);
            resource.StorageCapacity.Should().Be(1000.0f);
            resource.DecayRate.Should().Be(0.0f);
            resource.IsTradeableDefault.Should().BeTrue();
        }

        /// <summary>
        /// Test that parameterized constructor with all parameters sets properties correctly.
        /// </summary>
        [Fact]
        public void FullParameterizedConstructor_SetsCoreProperties()
        {
            var resource = new ResourceDefinition(
                "food",
                "Food",
                "Basic sustenance for population",
                2.5f,
                500.0f,
                0.1f,
                true);

            resource.Id.Should().Be("food");
            resource.Name.Should().Be("Food");
            resource.Description.Should().Be("Basic sustenance for population");
            resource.ProductionRate.Should().Be(2.5f);
            resource.StorageCapacity.Should().Be(500.0f);
            resource.DecayRate.Should().Be(0.1f);
            resource.IsTradeableDefault.Should().BeTrue();
        }

        /// <summary>
        /// Test that parameterized constructor throws when Id is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullId_ThrowsArgumentNullException()
        {
            Action act = () => new ResourceDefinition(
                null!,
                "Food",
                "Description",
                1.0f,
                1000.0f,
                0.0f,
                true);

            act.Should().Throw<ArgumentNullException>().WithParameterName("id");
        }

        /// <summary>
        /// Test that parameterized constructor throws when Name is null.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullName_ThrowsArgumentNullException()
        {
            Action act = () => new ResourceDefinition(
                "food",
                null!,
                "Description",
                1.0f,
                1000.0f,
                0.0f,
                true);

            act.Should().Throw<ArgumentNullException>().WithParameterName("name");
        }

        /// <summary>
        /// Test that parameterized constructor handles null Description by converting to empty string.
        /// </summary>
        [Fact]
        public void ParameterizedConstructor_WithNullDescription_ConvertsToEmptyString()
        {
            var resource = new ResourceDefinition(
                "food",
                "Food",
                null!,
                1.0f,
                1000.0f,
                0.0f,
                true);

            resource.Description.Should().BeEmpty();
        }

        /// <summary>
        /// Test that Validate() succeeds with valid ResourceDefinition.
        /// </summary>
        [Fact]
        public void Validate_WithValidDefinition_ReturnsSuccess()
        {
            var resource = new ResourceDefinition(
                "wood",
                "Wood",
                "Building material",
                1.5f,
                800.0f,
                0.05f,
                true);

            var result = resource.Validate();

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
            var resource = new ResourceDefinition { Id = id, Name = "Valid Name" };

            var result = resource.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "id");
            result.Errors.Should().Contain(e => e.Rule == "non_empty");
        }

        /// <summary>
        /// Test that Validate() fails with blank Name.
        /// </summary>
        [Theory]
        [InlineData(null)]
        [InlineData("")]
        [InlineData("   ")]
        public void Validate_WithBlankName_ReturnsError(string? name)
        {
            var resource = new ResourceDefinition { Id = "food", Name = name };

            var result = resource.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "name");
            result.Errors.Should().Contain(e => e.Rule == "non_empty");
        }

        /// <summary>
        /// Test that Validate() fails with negative StorageCapacity.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeStorageCapacity_ReturnsError()
        {
            var resource = new ResourceDefinition
            {
                Id = "food",
                Name = "Food",
                StorageCapacity = -100.0f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "storage_capacity");
            result.Errors.Should().Contain(e => e.Rule == "min-value");
        }

        /// <summary>
        /// Test that Validate() fails with negative ProductionRate.
        /// </summary>
        [Fact]
        public void Validate_WithNegativeProductionRate_ReturnsError()
        {
            var resource = new ResourceDefinition
            {
                Id = "food",
                Name = "Food",
                ProductionRate = -0.5f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "production_rate");
            result.Errors.Should().Contain(e => e.Rule == "min-value");
        }

        /// <summary>
        /// Test that DecayRate can be zero (no decay).
        /// </summary>
        [Fact]
        public void DecayRate_WithZero_IsAllowed()
        {
            var resource = new ResourceDefinition
            {
                Id = "stone",
                Name = "Stone",
                DecayRate = 0.0f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that DecayRate can be negative (meaning no decay behavior).
        /// </summary>
        [Fact]
        public void DecayRate_WithNegative_IsAllowed()
        {
            var resource = new ResourceDefinition
            {
                Id = "stone",
                Name = "Stone",
                DecayRate = -0.1f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() accepts zero StorageCapacity.
        /// </summary>
        [Fact]
        public void Validate_WithZeroStorageCapacity_ReturnsSuccess()
        {
            var resource = new ResourceDefinition
            {
                Id = "test-resource",
                Name = "Test Resource",
                StorageCapacity = 0.0f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() accepts zero ProductionRate.
        /// </summary>
        [Fact]
        public void Validate_WithZeroProductionRate_ReturnsSuccess()
        {
            var resource = new ResourceDefinition
            {
                Id = "test-resource",
                Name = "Test Resource",
                ProductionRate = 0.0f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeTrue();
        }

        /// <summary>
        /// Test that Validate() detects multiple errors at once.
        /// </summary>
        [Fact]
        public void Validate_WithMultipleErrors_ReturnsAllErrors()
        {
            var resource = new ResourceDefinition
            {
                Id = "",
                Name = "",
                ProductionRate = -1.0f,
                StorageCapacity = -500.0f
            };

            var result = resource.Validate();

            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(4);
            result.Errors.Should().Contain(e => e.Path == "id");
            result.Errors.Should().Contain(e => e.Path == "name");
            result.Errors.Should().Contain(e => e.Path == "production_rate");
            result.Errors.Should().Contain(e => e.Path == "storage_capacity");
        }

        /// <summary>
        /// Test that IsTradeableDefault can be set to false.
        /// </summary>
        [Fact]
        public void IsTradeableDefault_CanBeSetToFalse()
        {
            var resource = new ResourceDefinition(
                "secret-material",
                "Secret Material",
                "Non-tradeable resource",
                1.0f,
                500.0f,
                0.0f,
                false);

            resource.IsTradeableDefault.Should().BeFalse();
        }

        /// <summary>
        /// Test that all property setters work after construction.
        /// </summary>
        [Fact]
        public void PropertySetters_AllowModificationAfterConstruction()
        {
            var resource = new ResourceDefinition();
            resource.Id = "iron";
            resource.Name = "Iron";
            resource.Description = "Metal for tools and weapons";
            resource.ProductionRate = 0.8f;
            resource.StorageCapacity = 600.0f;
            resource.DecayRate = 0.0f;
            resource.IsTradeableDefault = true;

            var result = resource.Validate();

            result.IsValid.Should().BeTrue();
            resource.ProductionRate.Should().Be(0.8f);
            resource.StorageCapacity.Should().Be(600.0f);
        }
    }
}
