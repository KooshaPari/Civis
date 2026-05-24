using System;
using System.Collections.Generic;
using System.IO;
using DINOForge.SDK.Models;
using DINOForge.SDK.Validation;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for <see cref="JsonGuard"/> validation guard utility.
    /// </summary>
    public class JsonGuardUnitTests
    {
        [Fact]
        public void ValidateOrThrow_WithValidItem_DoesNotThrow()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "unit-test",
                DisplayName = "Test Unit",
                FactionId = "faction-a",
                Stats = new UnitStats { Hp = 10f }
            };

            // Act
            var act = () => JsonGuard.ValidateOrThrow(unit);

            // Assert
            act.Should().NotThrow();
        }

        [Fact]
        public void ValidateOrThrow_WithInvalidItem_ThrowsInvalidDataException()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "Test Unit",
                FactionId = "faction-a"
            };

            // Act
            var act = () => JsonGuard.ValidateOrThrow(unit);

            // Assert
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*failed semantic validation*");
        }

        [Fact]
        public void ValidateOrThrow_WithSource_IncludesSourceInMessage()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "Test Unit",
                FactionId = "faction-a"
            };
            const string source = "units/example.yaml";

            // Act
            var act = () => JsonGuard.ValidateOrThrow(unit, source);

            // Assert
            act.Should().Throw<InvalidDataException>()
                .WithMessage($"*UnitDefinition at '{source}'*");
        }

        [Fact]
        public void ValidateOrThrow_WithNonValidatableType_DoesNotThrow()
        {
            // Arrange
            object nonValidatable = new { Value = 42 };

            // Act
            var act = () => JsonGuard.ValidateOrThrow(nonValidatable);

            // Assert
            act.Should().NotThrow();
        }

        [Fact]
        public void ValidateOrThrow_WithMultipleErrors_IncludesAllInMessage()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "",
                DisplayName = "",
                FactionId = "",
                Stats = new UnitStats { Hp = 0f }
            };

            // Act
            var act = () => JsonGuard.ValidateOrThrow(unit);

            // Assert
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*id:*");
        }

        [Fact]
        public void TryValidate_WithValidItem_ReturnsTrueAndNullError()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "barracks",
                DisplayName = "Barracks",
                Health = 100
            };

            // Act
            var result = JsonGuard.TryValidate(building, out var errorSummary);

            // Assert
            result.Should().BeTrue();
            errorSummary.Should().BeNull();
        }

        [Fact]
        public void TryValidate_WithInvalidItem_ReturnsFalseAndErrorMessage()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "",
                DisplayName = "Barracks",
                Health = -10
            };

            // Act
            var result = JsonGuard.TryValidate(building, out var errorSummary);

            // Assert
            result.Should().BeFalse();
            errorSummary.Should().NotBeNull();
            errorSummary.Should().Contain("id");
        }

        [Fact]
        public void TryValidate_WithNonValidatableType_ReturnsTrueAndNullError()
        {
            // Arrange
            var nonValidatable = new System.Collections.Generic.Dictionary<string, int> { { "key", 1 } };

            // Act
            var result = JsonGuard.TryValidate(nonValidatable, out var errorSummary);

            // Assert
            result.Should().BeTrue();
            errorSummary.Should().BeNull();
        }

        [Fact]
        public void TryValidate_WithMultipleErrors_CombinesAllErrorsInMessage()
        {
            // Arrange
            var weapon = new WeaponDefinition
            {
                Id = "",
                DisplayName = "",
                BaseDamage = -5f
            };

            // Act
            var result = JsonGuard.TryValidate(weapon, out var errorSummary);

            // Assert
            result.Should().BeFalse();
            errorSummary.Should().NotBeNull();
            errorSummary.Should().Contain("id");
            errorSummary.Should().Contain("display_name");
        }

        [Fact]
        public void TryValidate_WithInvalidItem_ErrorSummaryFormatSeparatedBySemicolon()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "",
                DisplayName = "",
                Health = -50
            };

            // Act
            var result = JsonGuard.TryValidate(building, out var errorSummary);

            // Assert
            result.Should().BeFalse();
            errorSummary.Should().Contain("; ");
        }

        [Fact]
        public void ValidateOrThrow_WithNullItem_DoesNotThrow()
        {
            // Act
            var act = () => JsonGuard.ValidateOrThrow<string>(null!);

            // Assert
            act.Should().NotThrow();
        }

        [Fact]
        public void TryValidate_WithNullItem_ReturnsTrueAndNullError()
        {
            // Act
            var result = JsonGuard.TryValidate<string>(null!, out var errorSummary);

            // Assert
            result.Should().BeTrue();
            errorSummary.Should().BeNull();
        }

        [Fact]
        public void ValidateOrThrow_ErrorMessageContainsFieldPath()
        {
            // Arrange
            var unit = new UnitDefinition
            {
                Id = "valid-id",
                DisplayName = "Valid Name",
                FactionId = "valid-faction",
                Stats = new UnitStats { Accuracy = 1.5f }
            };

            // Act
            var act = () => JsonGuard.ValidateOrThrow(unit);

            // Assert
            act.Should().Throw<InvalidDataException>()
                .WithMessage("*stats.accuracy*");
        }

        [Fact]
        public void TryValidate_ValidBuildingWithAllOptionalFields_ReturnsTrue()
        {
            // Arrange
            var building = new BuildingDefinition
            {
                Id = "hq",
                DisplayName = "Headquarters",
                Description = "Strategic command center",
                BuildingType = "command",
                Health = 200,
                VisualAsset = "prefab-hq",
                DefenseTags = new List<string> { "Fortified" }
            };

            // Act
            var result = JsonGuard.TryValidate(building, out var errorSummary);

            // Assert
            result.Should().BeTrue();
            errorSummary.Should().BeNull();
        }
    }
}
