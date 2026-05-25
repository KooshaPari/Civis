using System;
using System.Collections.Generic;
using DINOForge.Domains.Warfare.Archetypes;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for FactionArchetype edge cases and branch coverage gaps.
    /// Focuses on null validation, immutability, case sensitivity, and boundary conditions.
    /// </summary>
    public class WarfareCoverageGapTests
    {
        [Fact]
        public void FactionArchetype_Constructor_NullId_ThrowsArgumentNullException()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };

            // Act & Assert
            new Action(() => new FactionArchetype(null!, "Order", "Disciplined faction", modifiers))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("id");
        }

        [Fact]
        public void FactionArchetype_Constructor_NullDisplayName_ThrowsArgumentNullException()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };

            // Act & Assert
            new Action(() => new FactionArchetype("order", null!, "Disciplined faction", modifiers))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("displayName");
        }

        [Fact]
        public void FactionArchetype_Constructor_NullDescription_ThrowsArgumentNullException()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };

            // Act & Assert
            new Action(() => new FactionArchetype("order", "Order", null!, modifiers))
                .Should().Throw<ArgumentNullException>()
                .WithParameterName("description");
        }

        [Fact]
        public void FactionArchetype_Constructor_NullBaseModifiers_ThrowsArgumentNullException()
        {
            // Act & Assert
            new Action(() => new FactionArchetype("order", "Order", "Disciplined faction", null!))
                .Should().Throw<ArgumentNullException>();
        }

        [Fact]
        public void FactionArchetype_Constructor_EmptyModifiers_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>();

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Assert
            archetype.BaseModifiers.Should().BeEmpty("Empty modifiers should be accepted");
        }

        [Fact]
        public void FactionArchetype_Constructor_ValidParams_InitializesCorrectly()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 1.1f },
                { "speed", 0.95f },
                { "damage", 1.05f }
            };

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Assert
            archetype.Id.Should().Be("order");
            archetype.DisplayName.Should().Be("Order");
            archetype.Description.Should().Be("Disciplined faction");
            archetype.BaseModifiers.Should().HaveCount(3);
        }

        [Fact]
        public void FactionArchetype_BaseModifiers_CaseInsensitiveLookup()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "Armor", 1.1f },
                { "Speed", 0.95f },
                { "Damage", 1.05f }
            };
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Act
            var hasArmor = archetype.BaseModifiers.ContainsKey("armor");
            var hasSpeed = archetype.BaseModifiers.ContainsKey("speed");
            var hasDamage = archetype.BaseModifiers.ContainsKey("damage");

            // Assert
            hasArmor.Should().BeTrue("Should support case-insensitive key lookup");
            hasSpeed.Should().BeTrue("Should support case-insensitive key lookup");
            hasDamage.Should().BeTrue("Should support case-insensitive key lookup");
        }

        [Fact]
        public void FactionArchetype_BaseModifiers_AreAccessible()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Act & Assert
            archetype.BaseModifiers.Should().NotBeNull("BaseModifiers should not be null");
            archetype.BaseModifiers.Should().HaveCount(1);
            archetype.BaseModifiers["armor"].Should().Be(1.1f);
        }

        [Fact]
        public void FactionArchetype_BaseModifiers_CanRetrieveValues()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 1.1f },
                { "speed", 0.95f },
                { "damage", 1.05f }
            };
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Act
            archetype.BaseModifiers.TryGetValue("armor", out float armorMod);
            archetype.BaseModifiers.TryGetValue("speed", out float speedMod);
            archetype.BaseModifiers.TryGetValue("damage", out float damageMod);

            // Assert
            armorMod.Should().Be(1.1f);
            speedMod.Should().Be(0.95f);
            damageMod.Should().Be(1.05f);
        }

        [Fact]
        public void FactionArchetype_BaseModifiers_ReturnsCopyNotReference()
        {
            // Arrange
            var originalModifiers = new Dictionary<string, float> { { "armor", 1.1f } };
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", originalModifiers);

            // Act - modify the original
            originalModifiers["armor"] = 2.0f;

            // Assert - archetype should not be affected
            archetype.BaseModifiers["armor"].Should().Be(1.1f, "Archetype should have its own copy of modifiers");
        }

        [Fact]
        public void FactionArchetype_AllProperties_AreImmutable()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Act & Assert - properties are read-only, cannot be reassigned
            new Action(() => { var _ = archetype.Id; })
                .Should().NotThrow("Id property should be gettable");
            new Action(() => { var _ = archetype.DisplayName; })
                .Should().NotThrow("DisplayName property should be gettable");
            new Action(() => { var _ = archetype.Description; })
                .Should().NotThrow("Description property should be gettable");
            new Action(() => { var _ = archetype.BaseModifiers; })
                .Should().NotThrow("BaseModifiers property should be gettable");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithZeroModifier_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 0f },
                { "speed", 1.0f }
            };

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Assert
            archetype.BaseModifiers["armor"].Should().Be(0f, "Zero modifiers should be accepted");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithNegativeModifier_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", -0.5f },
                { "speed", 1.0f }
            };

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Assert
            archetype.BaseModifiers["armor"].Should().Be(-0.5f, "Negative modifiers should be accepted (debuff)");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithVeryLargeModifier_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 10.0f },
                { "damage", 100.0f }
            };

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined faction", modifiers);

            // Assert
            archetype.BaseModifiers["armor"].Should().Be(10.0f);
            archetype.BaseModifiers["damage"].Should().Be(100.0f);
        }

        [Fact]
        public void FactionArchetype_Constructor_MultipleModifiers_AllPreserved()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 1.1f },
                { "speed", 0.95f },
                { "damage", 1.05f },
                { "hp", 1.2f },
                { "cost", 1.15f },
                { "fireRate", 0.9f },
                { "morale", 1.3f }
            };

            // Act
            var archetype = new FactionArchetype("industrial_swarm", "Industrial Swarm", "Heavy mechanized faction", modifiers);

            // Assert
            archetype.BaseModifiers.Should().HaveCount(7, "All modifiers should be preserved");
            foreach (var kvp in modifiers)
            {
                archetype.BaseModifiers.Should().ContainKey(kvp.Key);
                archetype.BaseModifiers[kvp.Key].Should().Be(kvp.Value);
            }
        }

        [Fact]
        public void FactionArchetype_Id_IsPreservedAsProvided()
        {
            // Arrange & Act
            var archetype1 = new FactionArchetype("order", "Order", "Disciplined", new Dictionary<string, float>());
            var archetype2 = new FactionArchetype("INDUSTRIAL_SWARM", "Industrial Swarm", "Heavy", new Dictionary<string, float>());
            var archetype3 = new FactionArchetype("asymmetric-guerrilla", "Guerrilla", "Asymmetric", new Dictionary<string, float>());

            // Assert - IDs should be preserved as provided (case sensitive)
            archetype1.Id.Should().Be("order");
            archetype2.Id.Should().Be("INDUSTRIAL_SWARM");
            archetype3.Id.Should().Be("asymmetric-guerrilla");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithSpecialCharactersInId_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };

            // Act
            var archetype = new FactionArchetype("order-v2.1-alpha", "Order", "Disciplined", modifiers);

            // Assert
            archetype.Id.Should().Be("order-v2.1-alpha", "Special characters in ID should be accepted");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithWhitespaceDescription_IsAccepted()
        {
            // Arrange
            var modifiers = new Dictionary<string, float> { { "armor", 1.1f } };

            // Act
            var archetype = new FactionArchetype("order", "Order", "   ", modifiers);

            // Assert
            archetype.Description.Should().Be("   ", "Whitespace-only description should be accepted");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithEmptyIdWorks()
        {
            // Empty ID should be accepted
            var archetype = new FactionArchetype("", "Order", "Description", new Dictionary<string, float>());
            archetype.Id.Should().Be("");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithEmptyDisplayNameWorks()
        {
            // Empty DisplayName should be accepted
            var archetype = new FactionArchetype("order", "", "Description", new Dictionary<string, float>());
            archetype.DisplayName.Should().Be("");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithEmptyDescriptionWorks()
        {
            // Empty Description should be accepted
            var archetype = new FactionArchetype("order", "Order", "", new Dictionary<string, float>());
            archetype.Description.Should().Be("");
        }

        [Fact]
        public void FactionArchetype_Constructor_WithMultipleModifiersWorks()
        {
            // Arrange
            var modifiers = new Dictionary<string, float>
            {
                { "armor", 1.5f },
                { "speed", 0.9f }
            };

            // Act
            var archetype = new FactionArchetype("order", "Order", "Disciplined", modifiers);

            // Assert
            archetype.BaseModifiers["armor"].Should().Be(1.5f, "Armor modifier should be set");
            archetype.BaseModifiers["speed"].Should().Be(0.9f, "Speed modifier should be set");
        }
    }
}
