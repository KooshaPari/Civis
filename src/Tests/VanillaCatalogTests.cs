#nullable enable
using DINOForge.Runtime.Bridge;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests;

/// <summary>
/// Unit tests for the non-ECS parts of <see cref="VanillaCatalog"/> and
/// <see cref="VanillaEntityInfo"/> — testable without a running game.
///
/// VanillaCatalog.Build() requires Unity.Entities.EntityManager and belongs in
/// game-launch tests (GameLaunchPackTests.cs). These tests cover:
/// - Pre-build catalog state (all lists empty, IsBuilt = false)
/// - FindById on an unbuilt catalog
/// - VanillaEntityInfo construction and string representation
/// </summary>
public class VanillaCatalogTests
{
    // ─── Pre-build state ──────────────────────────────────────────────────────

    [Fact]
    public void NewCatalog_IsBuilt_IsFalse()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.IsBuilt.Should().BeFalse();
    }

    [Fact]
    public void NewCatalog_Units_IsEmpty()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.Units.Should().BeEmpty();
    }

    [Fact]
    public void NewCatalog_Buildings_IsEmpty()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.Buildings.Should().BeEmpty();
    }

    [Fact]
    public void NewCatalog_Projectiles_IsEmpty()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.Projectiles.Should().BeEmpty();
    }

    [Fact]
    public void NewCatalog_Other_IsEmpty()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.Other.Should().BeEmpty();
    }

    [Fact]
    public void FindById_UnbuiltCatalog_ReturnsNull()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.FindById("vanilla:melee_unit").Should().BeNull();
    }

    [Fact]
    public void FindById_Null_ReturnsNull()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.FindById(null!).Should().BeNull();
    }

    [Fact]
    public void FindById_EmptyString_ReturnsNull()
    {
        VanillaCatalog catalog = new VanillaCatalog();
        catalog.FindById("").Should().BeNull();
    }

    // ─── VanillaEntityInfo ────────────────────────────────────────────────────

    [Fact]
    public void VanillaEntityInfo_Constructor_SetsAllProperties()
    {
        string[] components = { "Components.Health", "Components.MeleeUnit" };
        VanillaEntityInfo info = new VanillaEntityInfo("vanilla:melee_unit", components, 42, "unit");

        info.InferredId.Should().Be("vanilla:melee_unit");
        info.ComponentTypes.Should().BeEquivalentTo(components);
        info.EntityCount.Should().Be(42);
        info.Category.Should().Be("unit");
    }

    [Fact]
    public void VanillaEntityInfo_ToString_ContainsInferredIdAndCounts()
    {
        VanillaEntityInfo info = new VanillaEntityInfo("vanilla:farm", new[] { "C1", "C2", "C3" }, 7, "building");
        string str = info.ToString();
        str.Should().Contain("vanilla:farm");
        str.Should().Contain("7");
    }

    [Fact]
    public void VanillaEntityInfo_EmptyComponents_ZeroEntityCount()
    {
        VanillaEntityInfo info = new VanillaEntityInfo("vanilla:unknown", System.Array.Empty<string>(), 0, "other");
        info.ComponentTypes.Should().BeEmpty();
        info.EntityCount.Should().Be(0);
    }

    [Fact]
    public void VanillaEntityInfo_CategoryIsPreserved()
    {
        foreach (string cat in new[] { "unit", "building", "projectile", "resource", "other" })
        {
            VanillaEntityInfo info = new VanillaEntityInfo($"vanilla:{cat}", System.Array.Empty<string>(), 1, cat);
            info.Category.Should().Be(cat);
        }
    }
}
