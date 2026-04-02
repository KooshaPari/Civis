#nullable enable
using System.Linq;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-002: warfare-starwars pack loads its full catalog in the live game.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchPackTests(GameLaunchFixture fixture)
{
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task WarfareStarwars_Loads28Units_InLiveCatalog()
    {
        CatalogSnapshot catalog = await fixture.Client!.GetCatalogAsync();

        catalog.Units.Should().NotBeEmpty("loaded packs should have registered units");

        int totalUnits = catalog.Units.Sum(u => u.EntityCount);
        totalUnits.Should().Be(28,
            "warfare-starwars defines 14 Republic units + 14 CIS units");
    }

    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task WarfareStarwars_IsListedInLoadedPacks()
    {
        GameStatus status = await fixture.Client!.StatusAsync();
        status.LoadedPacks.Should().Contain("warfare-starwars",
            "the warfare-starwars pack should be active after bootstrap");
    }
}
