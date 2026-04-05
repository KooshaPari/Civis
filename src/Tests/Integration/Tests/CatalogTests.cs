#nullable enable
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Integration.Fixtures;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Tests that the game catalog contains expected entity types.
/// These tests use the GameFixture which launches a real game instance.
/// Tests skip gracefully if the game is not installed or the ECS world is not ready.
/// On CI (no game): tests return early and pass with zero assertions.
/// On self-hosted runner with game: tests run and verify catalog contents.
/// </summary>
[Collection("Game")]
[Trait("Category", "Integration")]
[Trait("RequiresGame", "true")]
public class CatalogTests
{
    private readonly GameFixture _fixture;

    /// <summary>Initializes a new instance of <see cref="CatalogTests"/>.</summary>
    public CatalogTests(GameFixture fixture) => _fixture = fixture;

    /// <summary>
    /// Verifies that the catalog has unit entries.
    /// Conditionally skips via return guard when game is not available.
    /// </summary>
    [Fact]
    public async Task Catalog_HasUnits()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return;

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();
        catalog.Units.Should().NotBeEmpty(
            "the game should have unit archetypes");
    }

    /// <summary>
    /// Verifies that the catalog has building entries.
    /// Skips when game is not running or VanillaCatalog is not built.
    /// </summary>
    [Fact]
    public async Task Catalog_HasBuildings()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return;

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();
        catalog.Buildings.Should().NotBeEmpty(
            "the game should have building archetypes");
    }

    /// <summary>
    /// Verifies that the catalog has projectile entries.
    /// Skips when game is not running or VanillaCatalog is not built.
    /// </summary>
    [Fact]
    public async Task Catalog_HasProjectiles()
    {
        if (!_fixture.GameAvailable || !_fixture.Client.IsConnected)
            return;

        CatalogSnapshot catalog = await _fixture.Client.GetCatalogAsync();
        catalog.Projectiles.Should().NotBeEmpty(
            "the game should have projectile archetypes");
    }
}
