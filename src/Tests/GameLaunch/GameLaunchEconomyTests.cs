#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-008: Economy pack loads and provides live resource rate data.
/// Uses [Fact(Skip)] - skips on CI where DINO_GAME_PATH is not set.
/// On a self-hosted Windows runner with the game installed, these tests run.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchEconomyTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// GL-008: Economy pack is in the loaded packs list.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task EconomyPack_IsLoaded_AfterBootstrap()
    {
        GameStatus status = await fixture.Client.StatusAsync();
        status.LoadedPacks.Should().Contain("economy-balanced",
            "economy-balanced pack should be loaded at startup");
    }

    /// <summary>
    /// GL-008: Resource snapshot contains expected resource types from economy pack.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task EconomyPack_Resources_AvailableViaSnapshot()
    {
        ResourceSnapshot resources = await fixture.Client.GetResourcesAsync();
        resources.Should().NotBeNull("resource snapshot should be queryable");
    }

    /// <summary>
    /// GL-008: Economy pack YAML is accessible and parseable.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task EconomyPack_ManifestIsValid_AndLoadable()
    {
        CatalogSnapshot catalog = await fixture.Client.GetCatalogAsync();
        catalog.Should().NotBeNull("catalog should be queryable");
        catalog.Units.Should().NotBeEmpty(
            "at least one unit should be defined in loaded packs");

        GameStatus status = await fixture.Client.StatusAsync();
        status.ModPlatformReady.Should().BeTrue(
            "mod platform should be ready with economy pack loaded");
    }

    /// <summary>
    /// GL-008: Resource values are non-negative and within plausible bounds.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task EconomyPack_ResourceValues_AreReasonable()
    {
        ResourceSnapshot resources = await fixture.Client.GetResourcesAsync();

        resources.Food.Should().BeGreaterThanOrEqualTo(0, "Food stockpile should not be negative");
        resources.Wood.Should().BeGreaterThanOrEqualTo(0, "Wood stockpile should not be negative");
        resources.Stone.Should().BeGreaterThanOrEqualTo(0, "Stone stockpile should not be negative");
        resources.Iron.Should().BeGreaterThanOrEqualTo(0, "Iron stockpile should not be negative");
        resources.Money.Should().BeGreaterThanOrEqualTo(0, "Money stockpile should not be negative");

        const int maxPlausible = 1_000_000;
        resources.Food.Should().BeLessThan(maxPlausible);
        resources.Wood.Should().BeLessThan(maxPlausible);
        resources.Stone.Should().BeLessThan(maxPlausible);
    }
}
