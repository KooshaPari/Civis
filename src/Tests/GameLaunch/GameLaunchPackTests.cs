#nullable enable
using System;
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.SDK;
using DINOForge.SDK.Registry;
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

        // Ensure main menu + DFCanvas are active (cold start can report packs before UGUI is built).
        LoadSceneResult sceneResult = await fixture.Client.LoadSceneAsync(GameLaunchSceneNames.MainMenuBuildIndex);
        if (sceneResult.Success)
        {
            await Task.Delay(3000);
        }

        // DFCanvas/HudStrip may finish building after mod platform reports ready (alpha=0 strip is "exists" not "visible").
        UiWaitResult hudReady = await fixture.Client.WaitForUiAsync(
            "name=CountLabel",
            "exists",
            timeoutMs: 15_000);
        hudReady.Ready.Should().BeTrue(
            "HudStrip CountLabel should exist on DFCanvas after bootstrap (DFCanvas may lag mod platform)");

        UiActionResult hudLabel = await fixture.Client.QueryUiAsync("name=CountLabel");
        hudLabel.Success.Should().BeTrue("HudStrip CountLabel should be queryable after waitForUi");
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

        const string packId = WarfareStarwarsPackUnits.PackId;

        GameStatus status = await fixture.Client!.StatusAsync();
        status.LoadedPacks.Should().Contain(packId,
            "warfare-starwars must be active after bootstrap before counting registry units");

        // getCatalog returns ECS archetypes (EntityCount = live entities per archetype).
        // Summing EntityCount across the full world catalog is not a pack unit count (often 200+).
        CatalogSnapshot catalog = await fixture.Client.GetCatalogAsync();
        catalog.Units.Should().NotBeEmpty("ECS catalog should list unit archetypes after bootstrap");

        // Registry count: ContentLoader registers pack YAML into RegistryManager by SourcePackId.
        var registries = new RegistryManager();
        var loader = new ContentLoader(registries);
        loader.LoadPack(WarfareStarwarsPackPaths.ResolvePackRoot());
        loader.LastLoadErrorCount.Should().Be(0,
            "warfare-starwars pack YAML should load without errors: {0}",
            string.Join("; ", loader.LastLoadErrors));

        int packUnitCount = registries.Units.All.Values.Count(entry =>
            string.Equals(entry.SourcePackId, packId, StringComparison.OrdinalIgnoreCase));
        packUnitCount.Should().Be(WarfareStarwarsPackUnits.All.Length,
            "warfare-starwars defines 14 Republic units + 14 CIS units in the unit registry");

        foreach (string unitId in WarfareStarwarsPackUnits.All)
        {
            registries.Units.Contains(unitId).Should().BeTrue(
                "each warfare-starwars unit id should be registered: {0}", unitId);
        }
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
