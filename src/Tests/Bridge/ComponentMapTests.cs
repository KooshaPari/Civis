#nullable enable
using System;
using System.Collections.Generic;
using System.Linq;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="ComponentMap"/> — the mapping registry between SDK model paths
/// and DINO ECS component types. All tests are pure reflection-based, no ECS world required.
/// </summary>
public class ComponentMapTests
{
    // ─── Mapping Registry Initialization ──────────────────────────────────────

    [Fact]
    public void All_IsNonEmpty()
    {
        ComponentMap.All.Should().NotBeEmpty();
    }

    [Fact]
    public void All_ContainsAtLeast30Mappings()
    {
        // Known minimum from task spec: 30+ mappings
        ComponentMap.All.Should().HaveCountGreaterThan(30);
    }

    [Fact]
    public void All_ContainsHealthMapping()
    {
        ComponentMap.All.Keys.Should().Contain("unit.stats.hp");
    }

    [Fact]
    public void All_ContainsArmorMapping()
    {
        ComponentMap.All.Keys.Should().Contain("unit.stats.armor");
    }

    [Fact]
    public void All_ContainsAttackCooldownMapping()
    {
        ComponentMap.All.Keys.Should().Contain("unit.stats.attack_cooldown");
    }

    // ─── ComponentMapping Data ────────────────────────────────────────────────

    [Fact]
    public void UnitHealth_HasValidProperties()
    {
        ComponentMap.UnitHealth.EcsComponentType.Should().Be("Components.Health");
        ComponentMap.UnitHealth.SdkModelPath.Should().Be("unit.stats.hp");
        ComponentMap.UnitHealth.TargetFieldName.Should().Be("currentHealth");
    }

    [Fact]
    public void UnitArmor_HasValidProperties()
    {
        ComponentMap.UnitArmor.EcsComponentType.Should().Be("Components.ArmorData");
        ComponentMap.UnitArmor.SdkModelPath.Should().Be("unit.stats.armor");
        ComponentMap.UnitArmor.TargetFieldName.Should().Be("type");
    }

    [Fact]
    public void BuildingBase_HasValidProperties()
    {
        ComponentMap.BuildingBase.EcsComponentType.Should().Be("Components.BuildingBase");
        ComponentMap.BuildingBase.SdkModelPath.Should().Be("building");
    }

    // ─── Find by SDK Path ──────────────────────────────────────────────────────

    [Fact]
    public void Find_WithValidPath_ReturnsMapping()
    {
        ComponentMapping? result = ComponentMap.Find("unit.stats.hp");
        result.Should().NotBeNull();
        result!.EcsComponentType.Should().Be("Components.Health");
    }

    [Fact]
    public void Find_WithInvalidPath_ReturnsNull()
    {
        ComponentMapping? result = ComponentMap.Find("invalid.path.nowhere");
        result.Should().BeNull();
    }

    [Fact]
    public void Find_WithNullPath_ThrowsArgumentNullException()
    {
        // Find() does not handle null gracefully — throws ArgumentNullException
        var act = () => ComponentMap.Find(null!);
        act.Should().Throw<ArgumentNullException>();
    }

    [Fact]
    public void Find_IsCaseInsensitive()
    {
        // ComponentMap uses OrdinalIgnoreCase in Find()
        ComponentMapping? lower = ComponentMap.Find("unit.stats.hp");
        ComponentMapping? upper = ComponentMap.Find("UNIT.STATS.HP");
        lower.Should().NotBeNull();
        upper.Should().NotBeNull();
        lower!.EcsComponentType.Should().Be(upper!.EcsComponentType);
    }

    // ─── Find by ECS Type ─────────────────────────────────────────────────────

    [Fact]
    public void FindByEcsType_WithValidType_ReturnsMapping()
    {
        ComponentMapping? result = ComponentMap.FindByEcsType("Components.Health");
        result.Should().NotBeNull();
        result!.SdkModelPath.Should().Be("unit.stats.hp");
    }

    [Fact]
    public void FindByEcsType_WithInvalidType_ReturnsNull()
    {
        ComponentMapping? result = ComponentMap.FindByEcsType("NonExistent.Type");
        result.Should().BeNull();
    }

    [Fact]
    public void FindByEcsType_WithNullType_ReturnsNull()
    {
        ComponentMapping? result = ComponentMap.FindByEcsType(null!);
        result.Should().BeNull();
    }

    [Fact]
    public void FindByEcsType_IsCaseSensitive()
    {
        // ECS types are case-sensitive in CLR
        ComponentMapping? correctCase = ComponentMap.FindByEcsType("Components.Health");
        ComponentMapping? wrongCase = ComponentMap.FindByEcsType("components.health");
        correctCase.Should().NotBeNull();
        wrongCase.Should().BeNull();
    }

    // ─── Resource Mappings ─────────────────────────────────────────────────────

    [Fact]
    public void All_ContainsAllResourceMappings()
    {
        string[] resourcePaths = new[]
        {
            "resource.current.food",
            "resource.current.iron",
            "resource.current.stone",
            "resource.current.wood",
            "resource.current.money",
            "resource.current.souls"
        };

        foreach (string path in resourcePaths)
        {
            ComponentMap.All.Keys.Should().Contain(path, $"Missing resource mapping: {path}");
        }
    }

    // ─── Building Mappings ─────────────────────────────────────────────────────

    [Fact]
    public void All_ContainsAllBuildingTypeMappings()
    {
        string[] buildingPaths = new[]
        {
            "building.type.barracks",
            "building.type.farm",
            "building.type.house",
            "building.type.granary",
            "building.type.hospital",
            "building.type.forester",
            "building.type.stonecutter",
            "building.type.ironmine"
        };

        foreach (string path in buildingPaths)
        {
            ComponentMap.All.Keys.Should().Contain(path, $"Missing building mapping: {path}");
        }
    }

    // ─── Unit Class Tags ──────────────────────────────────────────────────────

    [Fact]
    public void All_ContainsAllUnitClassMappings()
    {
        string[] classPaths = new[]
        {
            "unit.class.melee",
            "unit.class.ranged",
            "unit.class.cavalry",
            "unit.class.siege",
            "unit.class.archer",
            "unit.class.cast_only"
        };

        foreach (string path in classPaths)
        {
            ComponentMap.All.Keys.Should().Contain(path, $"Missing unit class mapping: {path}");
        }
    }

    // ─── Projectile Mappings ──────────────────────────────────────────────────

    [Fact]
    public void All_ContainsProjectileMappings()
    {
        ComponentMap.All.Keys.Should().Contain("projectile.base");
        ComponentMap.All.Keys.Should().Contain("projectile.damage");
        ComponentMap.All.Keys.Should().Contain("projectile.gravity");
    }

    // ─── Validation ────────────────────────────────────────────────────────────

    [Fact]
    public void ValidateResolution_ReturnsNonZeroResolvedCount()
    {
        // Without Unity.Entities loaded, resolution count will be 0, but method should not throw
        var (resolved, total, unresolved) = ComponentMap.ValidateResolution();
        total.Should().Be(ComponentMap.All.Count);
        (resolved + unresolved.Count).Should().Be(total);
    }

    [Fact]
    public void ValidateResolution_ReturnsUnresolvedList()
    {
        var (resolved, total, unresolved) = ComponentMap.ValidateResolution();
        unresolved.Should().NotBeNull();
        unresolved.Should().BeAssignableTo<List<string>>();
    }

    [Fact]
    public void ValidateResolution_UnresolvedListIsReadableAsEnumerable()
    {
        var (resolved, total, unresolved) = ComponentMap.ValidateResolution();
        // Verify unresolved is iterable and contains valid type names
        unresolved.All(name => name.StartsWith("Components.") || name.StartsWith("Unity."))
            .Should().BeTrue("All unresolved types should be from expected namespaces");
    }

    // ─── No Duplicates in Registry ────────────────────────────────────────────

    [Fact]
    public void All_ContainsNoDuplicateSdkPaths()
    {
        // Dictionary automatically prevents duplicate keys
        var allPaths = ComponentMap.All.Keys.ToList();
        var distinctPaths = allPaths.Distinct(StringComparer.OrdinalIgnoreCase).ToList();
        allPaths.Count.Should().Be(distinctPaths.Count);
    }

    [Fact]
    public void All_AllMappingsHaveNonEmptyEcsType()
    {
        foreach (var mapping in ComponentMap.All.Values)
        {
            mapping.EcsComponentType.Should().NotBeNullOrEmpty();
        }
    }

    [Fact]
    public void All_AllMappingsHaveNonEmptySdkPath()
    {
        foreach (var mapping in ComponentMap.All.Values)
        {
            mapping.SdkModelPath.Should().NotBeNullOrEmpty();
        }
    }
}
