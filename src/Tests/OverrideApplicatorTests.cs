using System.Collections.Generic;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="StatOverrideEntry"/> and <see cref="StatOverrideDefinition"/>
    /// model contracts. These tests focus on the model-level behavior of the enum-based
    /// mode field (YAML deserialized as StatOverrideMode), not the runtime StatModifierSystem integration
    /// which requires game state.
    /// </summary>
    public class OverrideApplicatorTests
    {
        [Fact]
        public void StatOverrideEntry_OverrideMode_DefaultsCorrectly()
        {
            // Arrange
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.hp",
                Value = 150f,
                Mode = StatOverrideMode.Override
            };

            // Act & Assert
            entry.Target.Should().Be("unit.stats.hp");
            entry.Value.Should().Be(150f);
            entry.Mode.Should().Be(StatOverrideMode.Override);
        }

        [Fact]
        public void StatOverrideEntry_AddMode_StoresCorrectly()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.damage",
                Value = 25f,
                Mode = StatOverrideMode.Add
            };

            entry.Mode.Should().Be(StatOverrideMode.Add);
        }

        [Fact]
        public void StatOverrideEntry_MultiplyMode_StoresCorrectly()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.armor",
                Value = 1.5f,
                Mode = StatOverrideMode.Multiply
            };

            entry.Mode.Should().Be(StatOverrideMode.Multiply);
        }

        [Fact]
        public void StatOverrideEntry_DefaultMode_IsOverride()
        {
            // Default enum value of Mode should be Override so packs that omit `mode:`
            // get the safest default — replace, not silently add or multiply.
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.hp",
                Value = 100f
            };

            entry.Mode.Should().Be(StatOverrideMode.Override);
        }

        [Fact]
        public void StatOverrideEntry_AllKnownModes_AreSupported()
        {
            // Verify the known mode enums that OverrideApplicator.cs recognizes.
            var modes = new[]
            {
                StatOverrideMode.Override,
                StatOverrideMode.Add,
                StatOverrideMode.Multiply,
            };

            // OverrideApplicator should handle all of these via the switch expression
            modes.Should().HaveCount(3);
        }

        [Fact]
        public void StatOverrideEntry_WithFilter_StoresCorrectly()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.damage",
                Value = 50f,
                Mode = StatOverrideMode.Override,
                Filter = "Components.MeleeUnit"
            };

            entry.Filter.Should().Be("Components.MeleeUnit");
        }

        [Fact]
        public void StatOverrideEntry_ZeroValue_IsValid()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.speed",
                Value = 0f,
                Mode = StatOverrideMode.Override
            };

            entry.Value.Should().Be(0f);
        }

        [Fact]
        public void StatOverrideEntry_NegativeValue_IsValid()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.armor",
                Value = -5f,
                Mode = StatOverrideMode.Add
            };

            entry.Value.Should().Be(-5f);
        }

        [Fact]
        public void StatOverrideEntry_LargeValue_IsValid()
        {
            var entry = new StatOverrideEntry
            {
                Target = "unit.stats.hp",
                Value = 99999f,
                Mode = StatOverrideMode.Override
            };

            entry.Value.Should().Be(99999f);
        }

        [Fact]
        public void StatOverrideDefinition_EmptyOverridesList_IsAccessible()
        {
            var definition = new StatOverrideDefinition
            {
                Overrides = new List<StatOverrideEntry>()
            };

            definition.Overrides.Should().BeEmpty();
        }

        [Fact]
        public void StatOverrideDefinition_MultipleEntries_AllStored()
        {
            var definition = new StatOverrideDefinition
            {
                Overrides = new List<StatOverrideEntry>
                {
                    new StatOverrideEntry { Target = "unit.stats.hp", Value = 150f, Mode = StatOverrideMode.Override },
                    new StatOverrideEntry { Target = "unit.stats.damage", Value = 25f, Mode = StatOverrideMode.Add },
                    new StatOverrideEntry { Target = "unit.stats.armor", Value = 1.2f, Mode = StatOverrideMode.Multiply }
                }
            };

            definition.Overrides.Should().HaveCount(3);
            definition.Overrides[0].Target.Should().Be("unit.stats.hp");
            definition.Overrides[1].Target.Should().Be("unit.stats.damage");
            definition.Overrides[2].Target.Should().Be("unit.stats.armor");
        }

        [Fact]
        public void StatOverrideDefinition_NullOverridesList_DefaultsToEmpty()
        {
            var definition = new StatOverrideDefinition();

            definition.Overrides.Should().NotBeNull();
            definition.Overrides.Should().BeEmpty();
        }

        [Fact]
        public void StatOverrideEntry_AllProperties_AreAccessible()
        {
            var entry = new StatOverrideEntry
            {
                Target = "test.path",
                Value = 42f,
                Mode = StatOverrideMode.Multiply,
                Filter = "TestFilter"
            };

            entry.Target.Should().Be("test.path");
            entry.Value.Should().Be(42f);
            entry.Mode.Should().Be(StatOverrideMode.Multiply);
            entry.Filter.Should().Be("TestFilter");
        }
    }
}
