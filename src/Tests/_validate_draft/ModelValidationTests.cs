using FluentAssertions;
using Xunit;
using DINOForge.SDK.Models;

namespace DINOForge.Tests
{
    /// <summary>
    /// Unit tests for validation of high-risk SDK model classes.
    /// </summary>
    public class ModelValidationTests
    {
        [Fact]
        public void ResourceCost_NegativeGold_ReturnsValidationError()
        {
            // Arrange
            var cost = new ResourceCost { Gold = -5000 };

            // Act
            var result = cost.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Message.Should().Contain("Gold").And.Contain("negative");
            result.Errors[0].Rule.Should().Be("min-value");
        }

        [Fact]
        public void ResourceCost_AllNonNegative_IsValid()
        {
            // Arrange
            var cost = new ResourceCost { Food = 0, Wood = 10, Stone = 20, Iron = 30, Gold = 40, Population = 5 };

            // Act
            var result = cost.Validate();

            // Assert
            result.IsValid.Should().BeTrue();
            result.Errors.Should().BeEmpty();
        }

        [Fact]
        public void SpawnGroup_NegativeCount_ReturnsValidationError()
        {
            // Arrange
            var spawnGroup = new SpawnGroup { UnitId = "unit-1", Count = -5 };

            // Act
            var result = spawnGroup.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Message.Should().Contain("Count").And.Contain("must be >= 1");
            result.Errors[0].Rule.Should().Be("min-value");
        }

        [Fact]
        public void SquadDefinition_InvertedSize_ReturnsValidationError()
        {
            // Arrange
            var squad = new SquadDefinition { Id = "squad-1", DisplayName = "Test Squad", MinSize = 20, MaxSize = 10 };

            // Act
            var result = squad.Validate();

            // Assert
            result.IsValid.Should().BeFalse();
            result.Errors.Should().HaveCount(1);
            result.Errors[0].Message.Should().Contain("MaxSize").And.Contain("cannot be less than MinSize");
            result.Errors[0].Rule.Should().Be("constraint-violation");
        }
    }
}
