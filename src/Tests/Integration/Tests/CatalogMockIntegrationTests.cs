#nullable enable
using System.Collections.Generic;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for CatalogSnapshot model and catalog serialization.
/// These tests use mocks and don't require the game to be running.
/// They verify the catalog data model, serialization, and client parsing.
/// </summary>
[Trait("Category", "Catalog")]
public class CatalogMockIntegrationTests
{
    /// <summary>
    /// GIVEN a CatalogSnapshot with unit entries
    /// WHEN the units are accessed
    /// THEN the entries are returned correctly
    /// </summary>
    [Fact]
    public void CatalogSnapshot_WithUnits_ReturnsUnits()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Units = new List<CatalogEntry>
            {
                new() { InferredId = "infantry", Category = "unit", EntityCount = 10 },
                new() { InferredId = "cavalry", Category = "unit", EntityCount = 5 }
            }
        };

        // Act & Assert
        catalog.Units.Should().HaveCount(2);
        catalog.Units[0].InferredId.Should().Be("infantry");
        catalog.Units[1].InferredId.Should().Be("cavalry");
    }

    /// <summary>
    /// GIVEN a CatalogSnapshot with building entries
    /// WHEN the buildings are accessed
    /// THEN the entries are returned correctly
    /// </summary>
    [Fact]
    public void CatalogSnapshot_WithBuildings_ReturnsBuildings()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Buildings = new List<CatalogEntry>
            {
                new() { InferredId = "barracks", Category = "building", EntityCount = 2 },
                new() { InferredId = "tower", Category = "building", EntityCount = 1 }
            }
        };

        // Act & Assert
        catalog.Buildings.Should().HaveCount(2);
        catalog.Buildings[0].InferredId.Should().Be("barracks");
    }

    /// <summary>
    /// GIVEN a CatalogSnapshot with projectile entries
    /// WHEN the projectiles are accessed
    /// THEN the entries are returned correctly
    /// </summary>
    [Fact]
    public void CatalogSnapshot_WithProjectiles_ReturnsProjectiles()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Projectiles = new List<CatalogEntry>
            {
                new() { InferredId = "arrow", Category = "projectile", EntityCount = 100 },
                new() { InferredId = "cannonball", Category = "projectile", EntityCount = 50 }
            }
        };

        // Act & Assert
        catalog.Projectiles.Should().HaveCount(2);
    }

    /// <summary>
    /// GIVEN an empty CatalogSnapshot
    /// WHEN all properties are accessed
    /// THEN they return empty lists (not null)
    /// </summary>
    [Fact]
    public void CatalogSnapshot_Empty_ReturnsEmptyLists()
    {
        // Arrange
        var catalog = new CatalogSnapshot();

        // Act & Assert
        catalog.Units.Should().NotBeNull().And.BeEmpty();
        catalog.Buildings.Should().NotBeNull().And.BeEmpty();
        catalog.Projectiles.Should().NotBeNull().And.BeEmpty();
    }

    /// <summary>
    /// GIVEN a CatalogSnapshot with all entry types
    /// WHEN serialized to JSON
    /// THEN the JSON contains all entries
    /// </summary>
    [Fact]
    public void CatalogSnapshot_SerializesToJson_ContainsAllEntries()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Units = new List<CatalogEntry> { new() { InferredId = "u1", Category = "unit", EntityCount = 1 } },
            Buildings = new List<CatalogEntry> { new() { InferredId = "b1", Category = "building", EntityCount = 1 } },
            Projectiles = new List<CatalogEntry> { new() { InferredId = "p1", Category = "projectile", EntityCount = 1 } }
        };

        // Act
        string json = Newtonsoft.Json.JsonConvert.SerializeObject(catalog);
        var deserialized = Newtonsoft.Json.JsonConvert.DeserializeObject<CatalogSnapshot>(json);

        // Assert
        deserialized.Should().NotBeNull();
        deserialized!.Units.Should().HaveCount(1);
        deserialized.Buildings.Should().HaveCount(1);
        deserialized.Projectiles.Should().HaveCount(1);
    }

    /// <summary>
    /// GIVEN a CatalogEntry with all properties
    /// WHEN properties are accessed
    /// THEN they return the correct values
    /// </summary>
    [Fact]
    public void CatalogEntry_WithAllProperties_ReturnsCorrectValues()
    {
        // Arrange
        var entry = new CatalogEntry
        {
            InferredId = "test-unit",
            Category = "unit",
            ComponentCount = 8,
            EntityCount = 25
        };

        // Assert
        entry.InferredId.Should().Be("test-unit");
        entry.Category.Should().Be("unit");
        entry.ComponentCount.Should().Be(8);
        entry.EntityCount.Should().Be(25);
    }

    /// <summary>
    /// GIVEN a CatalogSnapshot with mixed entries
    /// WHEN filtering by category
    /// THEN only matching entries are returned
    /// </summary>
    [Fact]
    public void CatalogSnapshot_WithMixedEntries_CanFilterByCategory()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Units = new List<CatalogEntry>
            {
                new() { InferredId = "melee", Category = "unit", EntityCount = 10 },
                new() { InferredId = "ranged", Category = "unit", EntityCount = 10 }
            },
            Buildings = new List<CatalogEntry>
            {
                new() { InferredId = "wall", Category = "building", EntityCount = 5 }
            }
        };

        // Act
        var units = catalog.Units.FindAll(e => e.Category == "unit");
        var buildings = catalog.Buildings.FindAll(e => e.Category == "building");

        // Assert
        units.Should().HaveCount(2);
        buildings.Should().HaveCount(1);
    }

    /// <summary>
    /// GIVEN a CatalogSnapshot
    /// WHEN TotalEntityCount is calculated
    /// THEN it sums all entities across categories
    /// </summary>
    [Fact]
    public void CatalogSnapshot_TotalEntities_SumsAllCategories()
    {
        // Arrange
        var catalog = new CatalogSnapshot
        {
            Units = new List<CatalogEntry>
            {
                new() { InferredId = "u1", Category = "unit", EntityCount = 10 },
                new() { InferredId = "u2", Category = "unit", EntityCount = 20 }
            },
            Buildings = new List<CatalogEntry>
            {
                new() { InferredId = "b1", Category = "building", EntityCount = 5 }
            }
        };

        // Act
        var totalUnits = catalog.Units.Sum(e => e.EntityCount);
        var totalAll = totalUnits + catalog.Buildings.Sum(e => e.EntityCount);

        // Assert
        totalUnits.Should().Be(30);
        totalAll.Should().Be(35);
    }
}
