#nullable enable
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-002: warfare-starwars pack loads its full catalog in the live game.
/// GL-003: bootstrap reports a non-zero pack count via bridge status and UGUI HUD.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchPackTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// Bridge <c>status</c> reads <see cref="ModPlatform.GetLoadedPackIds"/>; HUD/mod menu
    /// show counts from <c>OnHudCountsChanged</c> / <c>ModMenuPresenter.Packs</c> after
    /// <c>PushLoadedPacksToUgui</c>. Guards the "0 packs display" regression when packs load.
    /// </summary>
    [SkippableFact]
    public async Task Bootstrap_LoadedPackCount_IsGreaterThanZero_ViaStatusAndHud()
    {
        fixture.SkipIfNotInitialized();

        GameStatus status = await fixture.Client!.StatusAsync();
        status.ModPlatformReady.Should().BeTrue("mod platform should be initialized after bootstrap");
        status.LoadedPacks.Should().NotBeEmpty(
            "status JSON should list pack IDs from ModPlatform._lastLoadResult.LoadedPacks");

        UiActionResult hudLabel = await fixture.Client.QueryUiAsync("name=CountLabel");
        hudLabel.Success.Should().BeTrue("HudStrip CountLabel should exist on DFCanvas after bootstrap");
        hudLabel.MatchedNode.Should().NotBeNull();
        hudLabel.MatchedNode!.Label.Should().NotBe("0 packs",
            "HUD strip label should reflect OnHudCountsChanged, not the Build() placeholder");
        hudLabel.MatchedNode.Label.Should().Contain("packs");
        hudLabel.MatchedNode.Label.Should().NotStartWith("0 ",
            "HUD strip must not show zero packs when status reports loaded packs");
    }

    [SkippableFact]
    public async Task WarfareStarwars_Loads28Units_InLiveCatalog()
    {
        fixture.SkipIfNotInitialized();

        CatalogSnapshot catalog = await fixture.Client!.GetCatalogAsync();

        catalog.Units.Should().NotBeEmpty("loaded packs should have registered units");

        int totalUnits = catalog.Units.Sum(u => u.EntityCount);
        totalUnits.Should().Be(28,
            "warfare-starwars defines 14 Republic units + 14 CIS units");
    }

    [SkippableFact]
    public async Task WarfareStarwars_IsListedInLoadedPacks()
    {
        fixture.SkipIfNotInitialized();

        GameStatus status = await fixture.Client!.StatusAsync();
        status.LoadedPacks.Should().Contain("warfare-starwars",
            "the warfare-starwars pack should be active after bootstrap");
    }
}
