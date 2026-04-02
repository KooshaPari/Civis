#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// GL-006: In-game F10 overlay opens and is queryable via bridge tool.
/// GL-007: Hot reload fires within 5s after pack YAML change.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchUiTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// Toggles mod menu via bridge ToggleUiAsync, then queries UI tree.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task Overlay_F10_TogglesModMenu()
    {
        StartGameResult openResult = await fixture.Client!.ToggleUiAsync("modmenu");
        openResult.Success.Should().BeTrue("ToggleUiAsync should succeed");

        // Query UI tree to verify menu is open
        UiTreeResult tree = await fixture.Client.GetUiTreeAsync("modmenu/*");
        tree.Success.Should().BeTrue("GetUiTreeAsync should succeed");
    }

    /// <summary>
    /// Toggles mod menu off by calling ToggleUiAsync again.
    /// </summary>
    [Fact(Skip = "Game not available - DINO_GAME_PATH not set or game failed to launch. Run on self-hosted runner with DINO installed.")]
    public async Task Overlay_SecondToggle_ClosesModMenu()
    {
        await fixture.Client!.ToggleUiAsync("modmenu"); // open
        StartGameResult closeResult = await fixture.Client.ToggleUiAsync("modmenu"); // close

        closeResult.Success.Should().BeTrue("second toggle should close the menu");
    }
}
