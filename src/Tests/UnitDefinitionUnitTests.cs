using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="UnitDefinition"/> Validate() and construction contracts.
    /// </summary>
    public class UnitDefinitionUnitTests
    {
        [Fact]
        public void Constructor_DefaultValues_InitializesCorrectly()
        {
            // Arrange & Act
            var unit = new UnitDefinition();

            // Assert
            unit.Id.Should().Be("");
            unit.DisplayName.Should().Be("");
            unit.FactionId.Should().Be("");
            unit.Description.Should().BeNull();
            unit.Stats.Should().NotBeNull();
            unit.DefenseTags.Should().BeEmpty();
            unit.BehaviorTags.Should().BeEmpty();
        }

        [Fact]
        public void Validate_WithValidData_ReturnsSuccess()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f, Accuracy = 0.8f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void Validate_MissingId_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f }
            };

            // Act
            var result = unit.Validate();

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
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = null!,
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("display_name");
        }

        [Fact]
        public void Validate_MissingFactionId_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "  ",
                Stats = new UnitStats { Hp = 10f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("faction_id");
        }

        [Fact]
        public void Validate_InvalidHpZero_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 0f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("stats.hp");
            result.Errors[0].Message.Should().Contain("greater than 0");
        }

        [Fact]
        public void Validate_NegativeHp_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = -5f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().Contain(e => e.Path == "stats.hp");
        }

        [Fact]
        public void Validate_AccuracyAboveOne_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f, Accuracy = 1.5f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("stats.accuracy");
            result.Errors[0].Message.Should().Contain("between 0 and 1");
        }

        [Fact]
        public void Validate_AccuracyNegative_ReturnsFailure()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f, Accuracy = -0.1f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Path.Should().Be("stats.accuracy");
        }

        [Fact]
        public void Validate_MultipleErrors_ReturnsAllErrors()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "",
                FactionId = "",
                Stats = new UnitStats { Hp = 0f, Accuracy = 1.5f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(5); // id, display_name, faction_id, hp, accuracy
        }

        [Fact]
        public void Validate_ValidBoundaryAccuracy_ReturnsSuccess()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-test",
                DisplayName = "Test Unit",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 1f, Accuracy = 0.0f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_ValidBoundaryAccuracyOne_ReturnsSuccess()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-test",
                DisplayName = "Test Unit",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f, Accuracy = 1.0f }
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
        }

        [Fact]
        public void Validate_WithOptionalFields_ReturnsSuccess()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-rifle",
                DisplayName = "Rifleman",
                FactionId = "faction-a",
                Description = "A well-trained rifleman",
                Weapon = "weapon-rifle",
                VisualAsset = "prefab-rifleman",
                Stats = new UnitStats { Hp = 10f },
                VanillaMapping = "vanilla-unit-1"
            };

            // Act
            var result = unit.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }
    }
}
