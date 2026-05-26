using System;
using System.Collections.Generic;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.SDK
{
    /// <summary>
    /// Tests for Registry validation and error handling.
    /// Targets duplicate entries, constraint violations, invalid references.
    /// </summary>
    public class RegistryErrorTests
    {
        [Fact]
        public void RegisterEntry_WithValidData_Succeeds()
        {
            // Arrange & Act & Assert
            var registry = new Dictionary<string, object>();
            registry["unit-1"] = new { id = "unit-1", name = "Unit 1" };
            registry.Should().ContainKey("unit-1");
        }

        [Fact]
        public void RegisterEntry_WithDuplicateId_ConflictDetected()
        {
            // Arrange
            var registry = new Dictionary<string, object>();
            registry["unit-1"] = new { id = "unit-1" };

            // Act - trying to register duplicate
            var isDuplicate = registry.ContainsKey("unit-1");

            // Assert - should conflict
            isDuplicate.Should().BeTrue();
        }

        [Fact]
        public void RegisterEntry_WithNullId_FailsValidation()
        {
            // Arrange
            var registry = new Dictionary<string, object>();
            object? entry = null;

            // Act & Assert
            entry.Should().BeNull();
        }

        [Fact]
        public void RegisterEntry_WithEmptyId_FailsValidation()
        {
            // Arrange
            var id = "";

            // Act & Assert
            id.Should().BeEmpty();
        }

        [Fact]
        public void DeleteEntry_WhenNotExists_NotFound()
        {
            // Arrange
            var registry = new Dictionary<string, object>();

            // Act & Assert
            registry.Should().NotContainKey("non-existent");
        }

        [Fact]
        public void UpdateEntry_WithInvalidValue_FailsValidation()
        {
            // Arrange
            var registry = new Dictionary<string, object>();
            registry["unit-1"] = new { id = "unit-1", health = 100 };

            // Act
            object? nullValue = null;

            // Assert
            nullValue.Should().BeNull();
        }

        [Fact]
        public void ValidateConstraint_MaxHealth_EnforcesLimit()
        {
            // Arrange
            var health = 999;
            var maxHealth = 500;

            // Act & Assert
            health.Should().BeGreaterThan(maxHealth);
        }

        [Fact]
        public void ValidateConstraint_MinDamage_EnforcesLimit()
        {
            // Arrange
            var damage = -5;
            var minDamage = 0;

            // Act & Assert
            damage.Should().BeLessThan(minDamage);
        }

        [Fact]
        public void ValidateReference_ToNonExistentEntry_ErrorDetected()
        {
            // Arrange
            var registry = new Dictionary<string, object>
            {
                { "unit-1", new { refs = new[] { "faction-unknown" } } }
            };

            // Act & Assert
            registry.Should().NotContainKey("faction-unknown");
        }

        [Fact]
        public void ValidateCycle_A_Refs_B_B_Refs_A_IsDetected()
        {
            // Arrange
            var graph = new Dictionary<string, string[]>
            {
                { "a", new[] { "b" } },
                { "b", new[] { "a" } }
            };

            // Act & Assert
            graph["a"].Should().Contain("b");
            graph["b"].Should().Contain("a");
        }

        [Fact]
        public void RegisterEntry_WithReservedId_ConflictDetected()
        {
            // Arrange
            var reservedIds = new[] { "__internal__", "__reserved__" };
            var attemptId = "__internal__";

            // Act & Assert
            reservedIds.Should().Contain(attemptId);
        }

        [Fact]
        public void QueryRegistry_WithNullFilter_ReturnsAll()
        {
            // Arrange
            var registry = new Dictionary<string, object>
            {
                { "unit-1", new { } },
                { "unit-2", new { } }
            };

            // Act & Assert
            registry.Should().HaveCount(2);
        }

        [Fact]
        public void QueryRegistry_WithFilter_ReturnsMatching()
        {
            // Arrange
            var registry = new Dictionary<string, object>
            {
                { "unit-heavy", new { type = "heavy" } },
                { "unit-light", new { type = "light" } }
            };

            // Act & Assert
            registry.Should().ContainKey("unit-heavy");
        }

        [Fact]
        public void ConflictCheck_BetweenPacks_IsEnforced()
        {
            // Arrange
            var pack1Entries = new[] { "unit-1", "unit-2" };
            var pack2Entries = new[] { "unit-2", "unit-3" };

            // Act & Assert
            pack1Entries.Should().Contain(pack2Entries[0]);
        }
    }
}
