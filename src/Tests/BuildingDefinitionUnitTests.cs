using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="BuildingDefinition"/> Validate() and construction contracts.
    /// </summary>
    public class BuildingDefinitionUnitTests
    {
        [Fact]
        public void Constructor_DefaultValues_InitializesCorrectly()
        {
            // Arrange & Act
            var building = new BuildingDefinition();

            // Assert
            building.Id.Should().Be("");
            building.DisplayName.Should().Be("");
            building.Description.Should().BeNull();
            building.Health.Should().Be(0);
            building.Cost.Should().NotBeNull();
            building.Production.Should().BeEmpty();
            building.DefenseTags.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithValidData_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = "Main Barracks",
                Health = 100
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_MissingId_ReturnsFailure()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "",
                DisplayName = "Main Barracks",
                Health = 100
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("id");
            result.Errors[0].Message.Should().Contain("required");
        }

        [Fact]
        public void Validate_MissingDisplayName_ReturnsFailure()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = null!,
                Health = 100
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("display_name");
        }

        [Fact]
        public void Validate_WhitespaceDisplayName_ReturnsFailure()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = "   ",
                Health = 100
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "display_name");
        }

        [Fact]
        public void Validate_NegativeHealth_ReturnsFailure()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = "Main Barracks",
                Health = -50
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("health");
            result.Errors[0].Message.Should().Contain("non-negative");
        }

        [Fact]
        public void Validate_ZeroHealth_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = "Main Barracks",
                Health = 0
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_MultipleErrors_ReturnsAllErrors()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "",
                DisplayName = "",
                Health = -10
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(3); // id, display_name, health
        }

        [Fact]
        public void Validate_WithOptionalFields_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks-main",
                DisplayName = "Main Barracks",
                Description = "Central infantry training facility",
                BuildingType = "barracks",
                Health = 100,
                VisualAsset = "prefab-barracks"
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithProductionDictionary_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "factory",
                DisplayName = "Vehicle Factory",
                Health = 150,
                Production = new System.Collections.Generic.Dictionary<string, int> { { "tank", 1 }, { "apc", 2 } }
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithDefenseTags_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "fortress",
                DisplayName = "Fortress",
                Health = 250,
                DefenseTags = new System.Collections.Generic.List<string> { "Fortified", "AntiAir" }
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithAntiAirProperties_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "aa-tower",
                DisplayName = "AA Tower",
                Health = 80,
                DefenseTags = new System.Collections.Generic.List<string> { "AntiAir" },
                AntiAir = new BuildingAntiAirProperties { Range = 30f, DamageBonus = 2.0f }
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_LargeHealth_ReturnsSuccess()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "fortress-ultimate",
                DisplayName = "Ultimate Fortress",
                Health = 999999
            };

            // Act
            var result = building.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }
    }
}
