// Tests for PackStatMappings — the pure-C# data layer of the PackStatInjector bridge.
//
// PackStatInjector itself (the ECS execution layer) requires Unity.Entities.EntityManager
// and is excluded from CI builds without the game. PackStatMappings has no Unity dependency
// and is fully testable here.
//
// Full ECS integration (PackStatInjector.Apply with a live EntityManager) belongs in
// src/Tests/Integration/.

using System.Collections.Generic;
using DINOForge.Runtime.Bridge;
using DINOForge.SDK;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests
{
    /// <summary>
    /// Tests for <see cref="PackStatMappings"/>: vanilla_mapping resolution,
    /// stat path enumeration, and edge-case handling.
    /// </summary>
    public class PackStatInjectorTests
    {
        // ──────────────────────────────────────────────────────────────
        //  VanillaMappingToComponentType table coverage
        // ──────────────────────────────────────────────────────────────

        [Theory]
        [InlineData("militia",         "Components.MeleeUnit")]
        [InlineData("line_infantry",   "Components.MeleeUnit")]
        [InlineData("heavy_infantry",  "Components.MeleeUnit")]
        [InlineData("elite",           "Components.MeleeUnit")]
        [InlineData("hero",            "Components.MeleeUnit")]
        [InlineData("wall_defender",   "Components.MeleeUnit")]
        [InlineData("support",         "Components.MeleeUnit")]
        [InlineData("special",         "Components.MeleeUnit")]
        [InlineData("ranged_infantry", "Components.RangeUnit")]
        [InlineData("skirmisher",      "Components.RangeUnit")]
        [InlineData("scout",           "Components.RangeUnit")]
        [InlineData("cavalry",         "Components.CavalryUnit")]
        [InlineData("siege",           "Components.SiegeUnit")]
        public void VanillaMappingTable_KnownMappings_ReturnCorrectComponentType(
            string vanillaMapping, string expectedComponentType)
        {
            bool found = PackStatMappings.VanillaMappingToComponentType.TryGetValue(
                vanillaMapping, out string? actual);

            found.Should().BeTrue($"'{vanillaMapping}' should be in the mapping table");
            actual.Should().Be(expectedComponentType);
        }

        [Fact]
        public void VanillaMappingTable_AerialFighter_IsNullSkipSignal()
        {
            // aerial_fighter is present in the table but maps to null to signal
            // "handled by AerialSpawnSystem, skip here"
            bool found = PackStatMappings.VanillaMappingToComponentType.TryGetValue(
                "aerial_fighter", out string? componentType);

            found.Should().BeTrue("aerial_fighter must be registered so unknown-mapping logging is avoided");
            componentType.Should().BeNull("null signals intentional skip, not a missing entry");
        }

        // ──────────────────────────────────────────────────────────────
        //  TryResolveMapping
        // ──────────────────────────────────────────────────────────────

        [Theory]
        [InlineData("militia",   "Components.MeleeUnit")]
        [InlineData("MILITIA",   "Components.MeleeUnit")]   // case-insensitive
        [InlineData("Militia",   "Components.MeleeUnit")]
        [InlineData("cavalry",   "Components.CavalryUnit")]
        [InlineData("CAVALRY",   "Components.CavalryUnit")]
        [InlineData("siege",     "Components.SiegeUnit")]
        public void TryResolveMapping_ValidMappings_ReturnsTrueWithComponentType(
            string vanillaMapping, string expectedComponentType)
        {
            bool result = PackStatMappings.TryResolveMapping(vanillaMapping, out string? componentType);

            result.Should().BeTrue();
            componentType.Should().Be(expectedComponentType);
        }

        [Fact]
        public void TryResolveMapping_AerialFighter_ReturnsTrueWithNullComponentType()
        {
            // aerial_fighter is a known entry — returns true but componentType is null
            bool result = PackStatMappings.TryResolveMapping("aerial_fighter", out string? componentType);

            result.Should().BeTrue("aerial_fighter is registered in the table");
            componentType.Should().BeNull("null signals intentional skip by AerialSpawnSystem");
        }

        [Fact]
        public void TryResolveMapping_NullInput_ReturnsFalse()
        {
            bool result = PackStatMappings.TryResolveMapping(null, out string? componentType);

            result.Should().BeFalse();
            componentType.Should().BeNull();
        }

        [Fact]
        public void TryResolveMapping_EmptyString_ReturnsFalse()
        {
            bool result = PackStatMappings.TryResolveMapping("", out string? componentType);

            result.Should().BeFalse();
            componentType.Should().BeNull();
        }

        [Fact]
        public void TryResolveMapping_WhitespaceOnly_ReturnsFalse()
        {
            bool result = PackStatMappings.TryResolveMapping("   ", out string? componentType);

            result.Should().BeFalse();
            componentType.Should().BeNull();
        }

        [Fact]
        public void TryResolveMapping_UnknownMapping_ReturnsFalse()
        {
            bool result = PackStatMappings.TryResolveMapping("totally_unknown_unit_type", out _);

            result.Should().BeFalse("unknown strings must not resolve to any component type");
        }

        // ──────────────────────────────────────────────────────────────
        //  EnumerateStatPaths
        // ──────────────────────────────────────────────────────────────

        [Fact]
        public void EnumerateStatPaths_UnitWithHp200_EmitsHpPath()
        {
            UnitStats stats = new UnitStats { Hp = 200f };

            IEnumerable<(string SdkPath, float Value)> paths =
                PackStatMappings.EnumerateStatPaths(stats);

            paths.Should().Contain(p => p.SdkPath == "unit.stats.hp" && p.Value == 200f,
                "a unit with Hp=200 must emit 'unit.stats.hp' = 200");
        }

        [Fact]
        public void EnumerateStatPaths_UnitWithSpeed15_EmitsSpeedPath()
        {
            UnitStats stats = new UnitStats { Speed = 15f };

            IEnumerable<(string SdkPath, float Value)> paths =
                PackStatMappings.EnumerateStatPaths(stats);

            paths.Should().Contain(p => p.SdkPath == "unit.stats.speed" && p.Value == 15f);
        }

        [Fact]
        public void EnumerateStatPaths_ZeroStats_EmitsNoPaths()
        {
            // All stats at zero = use vanilla values, nothing to apply
            UnitStats stats = new UnitStats
            {
                Hp = 0f,
                Damage = 0f,
                Armor = 0f,
                Speed = 0f,
                Range = 0f,
                FireRate = 0f
            };

            IEnumerable<(string SdkPath, float Value)> paths =
                PackStatMappings.EnumerateStatPaths(stats);

            paths.Should().BeEmpty("zero-value stats should not produce any modification paths");
        }

        [Fact]
        public void EnumerateStatPaths_AllRelevantStats_EmitsAllExpectedPaths()
        {
            UnitStats stats = new UnitStats
            {
                Hp = 300f,
                Armor = 5f,
                Speed = 10f,
                FireRate = 2f,
                Range = 8f
            };

            List<(string SdkPath, float Value)> paths =
                new List<(string, float)>(PackStatMappings.EnumerateStatPaths(stats));

            paths.Should().Contain(p => p.SdkPath == "unit.stats.hp");
            paths.Should().Contain(p => p.SdkPath == "unit.stats.armor");
            paths.Should().Contain(p => p.SdkPath == "unit.stats.speed");
            paths.Should().Contain(p => p.SdkPath == "unit.stats.attack_cooldown");
            paths.Should().Contain(p => p.SdkPath == "unit.stats.range");
        }

        // ──────────────────────────────────────────────────────────────
        //  Table completeness
        // ──────────────────────────────────────────────────────────────

        [Fact]
        public void VanillaMappingTable_IsNotEmpty()
        {
            PackStatMappings.VanillaMappingToComponentType.Should().NotBeEmpty();
        }

        [Fact]
        public void VanillaMappingTable_ContainsAllDocumentedMappings()
        {
            // All vanilla_mapping values from the task spec must be present
            string[] requiredMappings = new[]
            {
                "militia", "line_infantry", "heavy_infantry", "ranged_infantry",
                "cavalry", "siege", "support", "scout", "elite", "hero",
                "wall_defender", "skirmisher", "special", "aerial_fighter"
            };

            foreach (string mapping in requiredMappings)
            {
                PackStatMappings.VanillaMappingToComponentType.Keys
                    .Should().Contain(mapping, $"'{mapping}' is a required vanilla_mapping value");
            }
        }

        [Fact]
        public void TryResolveMapping_AllDocumentedMappings_Resolve()
        {
            // Every vanilla_mapping value documented in the task spec must resolve
            string[] expectedMappings = new[]
            {
                "militia", "line_infantry", "heavy_infantry", "ranged_infantry",
                "cavalry", "siege", "support", "scout", "elite", "hero",
                "wall_defender", "skirmisher", "special", "aerial_fighter"
            };

            foreach (string mapping in expectedMappings)
            {
                bool found = PackStatMappings.TryResolveMapping(mapping, out _);
                found.Should().BeTrue($"'{mapping}' must be resolvable via TryResolveMapping");
            }
        }
    }
}
