namespace DINOForge.Tests;

using System;
using System.Linq;
using DINOForge.Runtime.Bridge;
using DINOForge.SDK.Models;
using FluentAssertions;
using Xunit;

/// <summary>
/// Tests for pure-C# Runtime bridge logic extracted via file-linking.
/// Tests the four bridge files that have zero Unity/BepInEx dependencies:
/// PackStatMappings, VanillaArchetypeMapper, ComponentMapping, ComponentMap.
/// These tests run in CI without requiring a Runtime project reference.
/// </summary>
public class RuntimeExtractionTests
{
    #region PackStatMappings Tests (9 tests)

    [Fact]
    public void TryResolveMapping_NullInput_ReturnsFalse()
    {
        // Act
        bool result = PackStatMappings.TryResolveMapping(null, out string? resolved);

        // Assert
        result.Should().BeFalse();
        resolved.Should().BeNull();
    }

    [Fact]
    public void TryResolveMapping_WhitespaceInput_ReturnsFalse()
    {
        // Act
        bool result = PackStatMappings.TryResolveMapping("   ", out string? resolved);

        // Assert
        result.Should().BeFalse();
        resolved.Should().BeNull();
    }

    [Fact]
    public void TryResolveMapping_UnknownMapping_ReturnsFalse()
    {
        // Act
        bool result = PackStatMappings.TryResolveMapping("unknown_mapping", out string? resolved);

        // Assert
        result.Should().BeFalse();
        resolved.Should().BeNull();
    }

    [Theory]
    [InlineData("militia", "Components.MeleeUnit")]
    [InlineData("ranged_infantry", "Components.RangeUnit")]
    [InlineData("cavalry", "Components.CavalryUnit")]
    [InlineData("siege", "Components.SiegeUnit")]
    public void TryResolveMapping_KnownMappings_ReturnsTrueWithType(string mapping, string expectedType)
    {
        // Act
        bool result = PackStatMappings.TryResolveMapping(mapping, out string? resolved);

        // Assert
        result.Should().BeTrue();
        resolved.Should().Be(expectedType);
    }

    [Fact]
    public void TryResolveMapping_AerialFighter_ReturnsTrueWithNullSentinel()
    {
        // Arrange & Act
        bool result = PackStatMappings.TryResolveMapping("aerial_fighter", out string? resolved);

        // Assert
        // aerial_fighter maps to null intentionally (skipped archetype)
        result.Should().BeTrue();
        resolved.Should().BeNull();
    }

    [Fact]
    public void EnumerateStatPaths_NullStats_ThrowsArgumentNullException()
    {
        // Act & Assert
#pragma warning disable CS8625 // Cannot convert null literal to non-nullable reference type
        var action = () => PackStatMappings.EnumerateStatPaths(null).ToList();
#pragma warning restore CS8625
        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void EnumerateStatPaths_AllZeroAndFireRateDefault_YieldsFireRateOnly()
    {
        // Arrange
        // FireRate defaults to 1.0f in UnitStats constructor
        var stats = new UnitStats { Hp = 0f, Speed = 0f, Armor = 0f, Damage = 0f, Range = 0f };

        // Act
        var paths = PackStatMappings.EnumerateStatPaths(stats).ToList();

        // Assert
        // Only FireRate (default 1.0f) should be yielded
        paths.Should().HaveCount(1);
        paths[0].SdkPath.Should().Be("unit.stats.attack_cooldown");
        paths[0].Value.Should().Be(1.0f);
    }

    [Fact]
    public void EnumerateStatPaths_HpAndFireRateDefault_YieldsTwoPaths()
    {
        // Arrange
        // FireRate defaults to 1.0f; setting Hp to 100f
        var stats = new UnitStats { Hp = 100f, Speed = 0f, Armor = 0f, Damage = 0f, Range = 0f };

        // Act
        var paths = PackStatMappings.EnumerateStatPaths(stats).ToList();

        // Assert
        paths.Should().HaveCount(2);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.hp" && p.Value == 100f);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.attack_cooldown" && p.Value == 1.0f);
    }

    [Fact]
    public void EnumerateStatPaths_AllNonZeroStats_YieldsFivePaths()
    {
        // Arrange
        // EnumerateStatPaths yields: Hp, Armor, Speed, FireRate, Range (NOT Damage)
        var stats = new UnitStats { Hp = 100f, Speed = 5f, Armor = 20f, Damage = 15f, Range = 10f, FireRate = 0.8f };

        // Act
        var paths = PackStatMappings.EnumerateStatPaths(stats).ToList();

        // Assert
        paths.Should().HaveCount(5);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.hp" && p.Value == 100f);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.speed" && p.Value == 5f);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.armor" && p.Value == 20f);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.attack_cooldown" && p.Value == 0.8f);
        paths.Should().Contain(p => p.SdkPath == "unit.stats.range" && p.Value == 10f);
    }

    #endregion

    #region VanillaArchetypeMapper Tests (9 tests)

    [Fact]
    public void MapUnitClassToComponentType_NullInput_ReturnsNull()
    {
        // Act
        string? result = VanillaArchetypeMapper.MapUnitClassToComponentType(null);

        // Assert
        result.Should().BeNull();
    }

    [Fact]
    public void MapUnitClassToComponentType_EmptyInput_ReturnsNull()
    {
        // Act
        string? result = VanillaArchetypeMapper.MapUnitClassToComponentType("");

        // Assert
        result.Should().BeNull();
    }

    [Fact]
    public void MapUnitClassToComponentType_UnknownClass_ReturnsNull()
    {
        // Act
        string? result = VanillaArchetypeMapper.MapUnitClassToComponentType("UnknownUnitClass");

        // Assert
        result.Should().BeNull();
    }

    [Theory]
    [InlineData("MilitiaLight", "Components.MeleeUnit")]
    [InlineData("Skirmisher", "Components.RangeUnit")]
    [InlineData("FastVehicle", "Components.CavalryUnit")]
    [InlineData("Artillery", "Components.SiegeUnit")]
    [InlineData("Archer", "Components.Archer")]
    public void MapUnitClassToComponentType_ValidClasses_ReturnsCorrectType(string unitClass, string expectedType)
    {
        // Act
        string? result = VanillaArchetypeMapper.MapUnitClassToComponentType(unitClass);

        // Assert
        result.Should().Be(expectedType);
    }

    [Fact]
    public void MapUnitClassToComponentType_IsCaseInsensitive()
    {
        // Act
        string? lowerResult = VanillaArchetypeMapper.MapUnitClassToComponentType("militialight");
        string? upperResult = VanillaArchetypeMapper.MapUnitClassToComponentType("MILITIALIGHT");
        string? mixedResult = VanillaArchetypeMapper.MapUnitClassToComponentType("MilItIaLiGhT");

        // Assert
        lowerResult.Should().Be("Components.MeleeUnit");
        upperResult.Should().Be("Components.MeleeUnit");
        mixedResult.Should().Be("Components.MeleeUnit");
    }

    [Theory]
    [InlineData("MilitiaLight")]
    [InlineData("Skirmisher")]
    [InlineData("FastVehicle")]
    public void IsSpawnable_ValidClass_ReturnsTrue(string unitClass)
    {
        // Act
        bool result = VanillaArchetypeMapper.IsSpawnable(unitClass);

        // Assert
        result.Should().BeTrue();
    }

    [Theory]
    [InlineData("InvalidClass")]
    [InlineData("")]
    public void IsSpawnable_InvalidClass_ReturnsFalse(string unitClass)
    {
        // Act
        bool result = VanillaArchetypeMapper.IsSpawnable(unitClass);

        // Assert
        result.Should().BeFalse();
    }

    [Fact]
    public void IsSpawnable_NullClass_ReturnsFalse()
    {
        // Act
#pragma warning disable CS8604 // Possible null reference argument
        bool result = VanillaArchetypeMapper.IsSpawnable(null);
#pragma warning restore CS8604

        // Assert
        result.Should().BeFalse();
    }

    [Fact]
    public void ValidateUnitClass_EmptyClass_ReturnsFalseWithError()
    {
        // Act
        bool result = VanillaArchetypeMapper.ValidateUnitClass("", out string? error);

        // Assert
        result.Should().BeFalse();
        error.Should().NotBeNullOrEmpty();
    }

    [Fact]
    public void ValidateUnitClass_UnknownClass_ReturnsFalseWithErrorContainingClassName()
    {
        // Act
        bool result = VanillaArchetypeMapper.ValidateUnitClass("UnknownClass", out string? error);

        // Assert
        result.Should().BeFalse();
        error.Should().Contain("UnknownClass");
    }

    #endregion

    #region ComponentMapping Tests (5 tests)

    [Fact]
    public void ComponentMapping_Constructor_SetsAllProperties()
    {
        // Arrange & Act
        var mapping = new ComponentMapping("Components.Health", "unit.stats.hp", "Unit health points");

        // Assert
        mapping.EcsComponentType.Should().Be("Components.Health");
        mapping.SdkModelPath.Should().Be("unit.stats.hp");
        mapping.Description.Should().Be("Unit health points");
    }

    [Fact]
    public void ComponentMapping_NullEcsType_ThrowsArgumentNullException()
    {
        // Act & Assert
        var action = () => new ComponentMapping(null!, "unit.stats.hp", "desc");
        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ComponentMapping_NullSdkPath_ThrowsArgumentNullException()
    {
        // Act & Assert
        var action = () => new ComponentMapping("Components.Health", null!, "desc");
        action.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void ComponentMapping_ToString_HasArrowFormat()
    {
        // Arrange
        var mapping = new ComponentMapping("Components.Health", "unit.stats.hp", "desc");

        // Act
        string result = mapping.ToString();

        // Assert
        result.Should().Contain("unit.stats.hp");
        result.Should().Contain("->");
        result.Should().Contain("Components.Health");
    }

    [Fact]
    public void ComponentMapping_OptionalParams_DefaultToNull()
    {
        // Arrange & Act
        var mapping = new ComponentMapping("Components.Health", "unit.stats.hp");

        // Assert
        mapping.Description.Should().BeNullOrEmpty();
        mapping.TargetFieldName.Should().BeNullOrEmpty();
    }

    #endregion

    #region ComponentMap Tests (7 tests)

    [Fact]
    public void ComponentMap_Find_KnownSdkPath_ReturnsMapping()
    {
        // Act
        var result = ComponentMap.Find("unit.stats.hp");

        // Assert
        result.Should().NotBeNull();
        result!.SdkModelPath.Should().Be("unit.stats.hp");
        result.EcsComponentType.Should().Be("Components.Health");
    }

    [Fact]
    public void ComponentMap_Find_UnknownPath_ReturnsNull()
    {
        // Act
        var result = ComponentMap.Find("unknown.path");

        // Assert
        result.Should().BeNull();
    }

    [Fact]
    public void ComponentMap_Find_IsCaseInsensitive()
    {
        // Act
        var lowerResult = ComponentMap.Find("unit.stats.hp");
        var upperResult = ComponentMap.Find("UNIT.STATS.HP");

        // Assert
        lowerResult.Should().NotBeNull();
        upperResult.Should().NotBeNull();
        lowerResult!.EcsComponentType.Should().Be(upperResult!.EcsComponentType);
    }

    [Fact]
    public void ComponentMap_FindByEcsType_KnownType_ReturnsFirstMatch()
    {
        // Act
        var result = ComponentMap.FindByEcsType("Components.Health");

        // Assert
        result.Should().NotBeNull();
        result!.EcsComponentType.Should().Be("Components.Health");
    }

    [Fact]
    public void ComponentMap_FindByEcsType_UnknownType_ReturnsNull()
    {
        // Act
        var result = ComponentMap.FindByEcsType("Components.Unknown");

        // Assert
        result.Should().BeNull();
    }

    [Fact]
    public void ComponentMap_All_ContainsExpectedPaths()
    {
        // Act
        var all = ComponentMap.All;

        // Assert
        all.Values.Should().Contain(m => m.SdkModelPath == "unit.stats.hp");
        all.Values.Should().Contain(m => m.SdkModelPath == "unit.stats.speed");
        all.Values.Should().Contain(m => m.SdkModelPath == "unit.stats.armor");
    }

    [Fact]
    public void ComponentMap_All_IsNonEmpty()
    {
        // Act
        var all = ComponentMap.All;

        // Assert
        all.Should().NotBeEmpty();
        all.Count.Should().BeGreaterThan(10);
    }

    #endregion
}
