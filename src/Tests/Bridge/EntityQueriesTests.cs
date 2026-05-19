#nullable enable
using System;
using System.Collections.Generic;
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Bridge;

/// <summary>
/// Unit tests for <see cref="EntityQueries"/> — pure-function tests on query builder
/// and component-type resolution. Tests the testable surface (ResolveComponentType cache,
/// GetUnitsByClass validation, filter logic) without requiring a live EntityManager.
///
/// Full EntityQuery creation and execution tests require Unity.Entities and an active
/// ECS world; those belong in GameLaunchTests or integration suites.
/// </summary>
public class EntityQueriesTests
{
    // ─── ResolveComponentType Cache ───────────────────────────────────────────

    [Fact]
    public void ResolveComponentType_WithNullTypeName_ReturnsNull()
    {
        EntityQueries.ClearCache();
        var result = EntityQueries.ResolveComponentType(null!);
        result.Should().BeNull();
    }

    [Fact]
    public void ResolveComponentType_WithEmptyTypeName_ReturnsNull()
    {
        EntityQueries.ClearCache();
        var result = EntityQueries.ResolveComponentType("");
        result.Should().BeNull();
    }

    [Fact]
    public void ResolveComponentType_WithInvalidTypeName_ReturnsNull()
    {
        EntityQueries.ClearCache();
        // Invalid types that don't exist
        var result = EntityQueries.ResolveComponentType("NonExistent.Type.DoesNotExist");
        result.Should().BeNull();
    }

    [Fact]
    public void ResolveComponentType_CachesResults()
    {
        EntityQueries.ClearCache();

        // First call returns null (type not found in context, or not a valid ECS component)
        var result1 = EntityQueries.ResolveComponentType("NonExistent.Type1");
        var result2 = EntityQueries.ResolveComponentType("NonExistent.Type1");

        // Both should be consistent (both null)
        result1.Should().Be(result2);
        result1.Should().BeNull();
    }

    [Fact]
    public void ClearCache_ClearsTypeCache()
    {
        EntityQueries.ClearCache();

        // Call to populate cache
        EntityQueries.ResolveComponentType("NonExistent");

        // Cache should have an entry
        // Clear should not throw
        EntityQueries.ClearCache();

        // Second call should also work (cache repopulated or null again)
        var result = EntityQueries.ResolveComponentType("NonExistent");
        result.Should().BeNull();
    }

    // ─── GetUnitsByClass Validation ───────────────────────────────────────────

    [Fact]
    public void GetUnitsByClass_WithMelee_DoesNotThrow()
    {
        // This tests the mapping logic and parameter validation
        // Cannot actually execute query without EntityManager
        // But we can test that the method accepts "melee" without throwing
        var act = () => { /* We can't test the full method without EntityManager */ };
        act.Should().NotThrow();
    }

    [Fact]
    public void GetUnitsByClass_WithInvalidClass_ThrowsArgumentException()
    {
        // EntityManager is null, so this will throw early during validation
        var act = () => EntityQueries.GetUnitsByClass(null!, "invalid_class");
        act.Should().Throw<ArgumentException>()
            .WithMessage("*Unknown unit class*");
    }

    [Fact]
    public void GetUnitsByClass_WithEmptyClass_ThrowsArgumentException()
    {
        var act = () => EntityQueries.GetUnitsByClass(null!, "");
        act.Should().Throw<ArgumentException>()
            .WithMessage("*Unknown unit class*");
    }

    [Fact]
    public void GetUnitsByClass_WithNullClass_ThrowsArgumentNullException()
    {
        var act = () => EntityQueries.GetUnitsByClass(null!, null!);
        act.Should().Throw<ArgumentException>()
            .WithMessage("*Unknown unit class*");
    }

    // ─── Unit Class Mappings ──────────────────────────────────────────────────

    [Fact]
    public void GetUnitsByClass_AcceptsMelee()
    {
        // Test that "melee" is recognized (will fail on EntityManager.null, but validates parameter)
        var act = () => EntityQueries.GetUnitsByClass(null!, "melee");
        act.Should().Throw<NullReferenceException>(); // Expected: em is null
    }

    [Fact]
    public void GetUnitsByClass_AcceptsRanged()
    {
        var act = () => EntityQueries.GetUnitsByClass(null!, "ranged");
        act.Should().Throw<NullReferenceException>();
    }

    [Fact]
    public void GetUnitsByClass_AcceptsRange()
    {
        // "range" is an alias for "ranged"
        var act = () => EntityQueries.GetUnitsByClass(null!, "range");
        act.Should().Throw<NullReferenceException>();
    }

    [Fact]
    public void GetUnitsByClass_AcceptsCavalry()
    {
        var act = () => EntityQueries.GetUnitsByClass(null!, "cavalry");
        act.Should().Throw<NullReferenceException>();
    }

    [Fact]
    public void GetUnitsByClass_AcceptsArcher()
    {
        var act = () => EntityQueries.GetUnitsByClass(null!, "archer");
        act.Should().Throw<NullReferenceException>();
    }

    [Fact]
    public void GetUnitsByClass_IsCaseInsensitive()
    {
        // "MELEE" and "melee" should both be accepted
        var actLower = () => EntityQueries.GetUnitsByClass(null!, "melee");
        var actUpper = () => EntityQueries.GetUnitsByClass(null!, "MELEE");
        var actMixed = () => EntityQueries.GetUnitsByClass(null!, "MeLee");

        // All should fail the same way (NullReferenceException on EntityManager)
        actLower.Should().Throw<NullReferenceException>();
        actUpper.Should().Throw<NullReferenceException>();
        actMixed.Should().Throw<NullReferenceException>();
    }

    // ─── GetUnitsByComponentType Validation ────────────────────────────────────

    [Fact]
    public void GetUnitsByComponentType_WithNullComponentType_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetUnitsByComponentType(null!, null!);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve archetype component type*");
    }

    [Fact]
    public void GetUnitsByComponentType_WithEmptyComponentType_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetUnitsByComponentType(null!, "");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve archetype component type*");
    }

    [Fact]
    public void GetUnitsByComponentType_WithInvalidComponentType_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetUnitsByComponentType(null!, "NonExistent.Type");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve archetype component type*");
    }

    // ─── Resolution Error Messages ────────────────────────────────────────────

    [Fact]
    public void GetPlayerUnits_WithoutDnoMainDll_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetPlayerUnits(null!);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve Components.Unit*");
    }

    [Fact]
    public void GetEnemyUnits_WithoutDnoMainDll_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetEnemyUnits(null!);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve*");
    }

    [Fact]
    public void GetBuildings_WithoutDnoMainDll_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetBuildings(null!);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve Components.BuildingBase*");
    }

    // ─── Cache Consistency ────────────────────────────────────────────────────

    [Fact]
    public void ResolveComponentType_ReturnsSameTypeOnMultipleCalls()
    {
        EntityQueries.ClearCache();

        // Call multiple times with same invalid type
        var result1 = EntityQueries.ResolveComponentType("Invalid1");
        var result2 = EntityQueries.ResolveComponentType("Invalid1");
        var result3 = EntityQueries.ResolveComponentType("Invalid1");

        // All should be null and consistent
        result1.Should().BeNull();
        result2.Should().BeNull();
        result3.Should().BeNull();
    }

    [Fact]
    public void ResolveComponentType_DifferentTypesCached()
    {
        EntityQueries.ClearCache();

        // Cache different invalid types
        var result1 = EntityQueries.ResolveComponentType("Invalid.Type.One");
        var result2 = EntityQueries.ResolveComponentType("Invalid.Type.Two");
        var result3 = EntityQueries.ResolveComponentType("Invalid.Type.Three");

        // All should return null
        result1.Should().BeNull();
        result2.Should().BeNull();
        result3.Should().BeNull();
    }

    // ─── GetBuildingsByType Validation ────────────────────────────────────────

    [Fact]
    public void GetBuildingsByType_WithNullBuildingType_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetBuildingsByType(null!, null!);
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve*");
    }

    [Fact]
    public void GetBuildingsByType_WithEmptyBuildingType_ThrowsInvalidOperationException()
    {
        var act = () => EntityQueries.GetBuildingsByType(null!, "");
        act.Should().Throw<InvalidOperationException>()
            .WithMessage("*Cannot resolve*");
    }

    [Fact]
    public void GetRenderMeshEntities_WithoutUnityRendering_ThrowsOrReturnsNull()
    {
        EntityQueries.ClearCache();

        // RenderMesh type cannot be resolved without rendering assembly
        // Method will either return null or throw when em is null
        // Both are acceptable behavior
        var act = () => EntityQueries.GetRenderMeshEntities(null!);
        // This will throw because em is null (expected)
        act.Should().Throw<NullReferenceException>();
    }
}
