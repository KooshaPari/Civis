#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests that the game catalog contains expected entity types.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
public class CatalogTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="CatalogTests"/>.</summary>
    public CatalogTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>Verifies that the catalog has unit entries.</summary>
    [Fact(Skip = "Catalog tests require the game to be running and fully initialized. " +
                 "Requires GameAvailable=true and VanillaCatalog.IsBuilt=true. " +
                 "Run manually with a live game instance.")]
    public async Task Catalog_HasUnits()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();

        catalog.Units.Should().NotBeEmpty("the game should have unit archetypes");
    }

    /// <summary>Verifies that the catalog has building entries.</summary>
    [Fact(Skip = "Catalog tests require the game to be running and fully initialized. " +
                 "Requires GameAvailable=true and VanillaCatalog.IsBuilt=true. " +
                 "Run manually with a live game instance.")]
    public async Task Catalog_HasBuildings()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();

        catalog.Buildings.Should().NotBeEmpty("the game should have building archetypes");
    }

    /// <summary>Verifies that the catalog has projectile entries.</summary>
    [Fact(Skip = "Catalog tests require the game to be running and fully initialized. " +
                 "Requires GameAvailable=true and VanillaCatalog.IsBuilt=true. " +
                 "Run manually with a live game instance.")]
    public async Task Catalog_HasProjectiles()
    {
        if (!_fixture.GameAvailable)
            return; // Game not available for integration testing

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();

        catalog.Projectiles.Should().NotBeEmpty("the game should have projectile archetypes");
    }
}
