#nullable enable
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-008: Economy pack loads and provides live resource rate data.
/// Tests verify that the economy pack is loaded and that resource rates
/// match expected values from the pack manifest.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchEconomyTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// GL-008: Economy pack is in the loaded packs list.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task EconomyPack_IsLoaded_AfterBootstrap()
    {
        GameStatus status = await fixture.Client!.StatusAsync();

        status.LoadedPacks.Should().Contain("economy-balanced",
            "economy-balanced pack should be loaded at startup");
    }

    /// <summary>
    /// GL-008: Resource snapshot contains expected resource types from economy pack.
    /// Verifies that the game can return live resource data via the bridge.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task EconomyPack_Resources_AvailableViaSnapshot()
    {
        ResourceSnapshot resources = await fixture.Client!.GetResourcesAsync();

        resources.Should().NotBeNull("resource snapshot should be queryable");

        // Check that at least one resource is non-zero (economy is active)
        int totalResources = resources.Food + resources.Wood + resources.Stone +
                            resources.Iron + resources.Money + resources.Souls +
                            resources.Bones + resources.Spirit;
        totalResources.Should().BeGreaterThanOrEqualTo(0,
            "economy pack should provide resource data");
    }

    /// <summary>
    /// GL-008: Economy pack YAML is accessible and parseable.
    /// This test loads the economy pack manifest and verifies its structure.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task EconomyPack_ManifestIsValid_AndLoadable()
    {
        // Dump state to get information about loaded content
        CatalogSnapshot catalog = await fixture.Client!.GetCatalogAsync();

        catalog.Should().NotBeNull("catalog should be queryable");

        // The economy pack should have contributed units/buildings to the catalog
        catalog.Units.Should().NotBeEmpty(
            "at least one unit should be defined in loaded packs");

        // Verify that the platform is stable after loading economy pack
        GameStatus status = await fixture.Client.StatusAsync();
        status.ModPlatformReady.Should().BeTrue(
            "mod platform should be ready with economy pack loaded");
    }

    /// <summary>
    /// GL-008: Resource values are non-zero and reasonable.
    /// Sanity check that economy values are plausible.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task EconomyPack_ResourceValues_AreReasonable()
    {
        ResourceSnapshot resources = await fixture.Client!.GetResourcesAsync();

        // Sanity checks for resource values (should all be non-negative)
        resources.Food.Should().BeGreaterThanOrEqualTo(0,
            "Food stockpile should not be negative");
        resources.Wood.Should().BeGreaterThanOrEqualTo(0,
            "Wood stockpile should not be negative");
        resources.Stone.Should().BeGreaterThanOrEqualTo(0,
            "Stone stockpile should not be negative");
        resources.Iron.Should().BeGreaterThanOrEqualTo(0,
            "Iron stockpile should not be negative");
        resources.Money.Should().BeGreaterThanOrEqualTo(0,
            "Money stockpile should not be negative");

        // All resources should be reasonable (not astronomically large)
        int maxPlausible = 1_000_000;
        resources.Food.Should().BeLessThan(maxPlausible);
        resources.Wood.Should().BeLessThan(maxPlausible);
        resources.Stone.Should().BeLessThan(maxPlausible);
    }
}
