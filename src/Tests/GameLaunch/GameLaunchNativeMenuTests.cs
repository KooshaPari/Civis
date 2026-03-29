#nullable enable
using System.Threading.Tasks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// NATIVE-001, NATIVE-002, NATIVE-003: Main menu "Mods" button injection and persistence.
/// These tests verify that the native menu receives our custom button and that
/// it survives scene transitions.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchNativeMenuTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// NATIVE-001: Main menu contains injected "Mods" button.
    /// This test queries the UI tree to find the Mods button in the main menu.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task MainMenu_HasModsButton_AfterInjection()
    {
        // Query the UI tree for any element with "Mods" text or button name
        DINOForge.Bridge.Protocol.UiActionResult result =
            await fixture.Client!.QueryUiAsync("button:contains('Mods')");

        result.Should().NotBeNull("Mods button should be queryable in UI");
        result.MatchCount.Should().BeGreaterThan(0,
            "UI should contain a 'Mods' button or element in the main menu");
    }

    /// <summary>
    /// NATIVE-002: Clicking the "Mods" button activates the mod menu overlay.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task MainMenu_ModsButton_OpensOverlay()
    {
        // Query UI to find the Mods button
        DINOForge.Bridge.Protocol.UiActionResult result =
            await fixture.Client!.QueryUiAsync("button:contains('Mods')");

        result.Should().NotBeNull("Mods button should be found in main menu");
        result.MatchCount.Should().BeGreaterThan(0, "Mods button should match the selector");

        // Click it
        DINOForge.Bridge.Protocol.UiActionResult clickResult =
            await fixture.Client.ClickUiAsync("button:contains('Mods')");

        clickResult.Success.Should().BeTrue(
            "clicking the Mods button should succeed");

        // Give UI time to respond
        await Task.Delay(300);

        // Check that the mod menu is now visible
        DINOForge.Bridge.Protocol.UiWaitResult waitResult =
            await fixture.Client.WaitForUiAsync("element:contains('ModMenu')", "visible", timeoutMs: 2000);

        waitResult.Ready.Should().BeTrue(
            "mod menu should be visible after Mods button click");
    }

    /// <summary>
    /// NATIVE-003: Mods button and menu survive scene transitions (main menu → gameplay).
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task MainMenu_ModsButton_PersistsAcrossSceneChanges()
    {
        // First, verify the button exists in main menu
        DINOForge.Bridge.Protocol.UiActionResult initialQuery =
            await fixture.Client!.QueryUiAsync("button:contains('Mods')");

        initialQuery.MatchCount.Should().BeGreaterThan(0,
            "Mods button should exist in main menu initially");

        // Transition to gameplay by starting a new game
        await fixture.Client.StartGameAsync();
        await Task.Delay(3000); // Allow scene load

        // Check that gameplay has loaded
        DINOForge.Bridge.Protocol.GameStatus status =
            await fixture.Client.StatusAsync();

        status.WorldReady.Should().BeTrue(
            "ECS world should be ready after scene transition");

        // Go back to main menu
        await fixture.Client.LoadSceneAsync("MainMenu");
        await Task.Delay(2000);

        // Verify Mods button still exists
        DINOForge.Bridge.Protocol.UiActionResult finalQuery =
            await fixture.Client.QueryUiAsync("button:contains('Mods')");

        finalQuery.MatchCount.Should().BeGreaterThan(0,
            "Mods button should persist after scene transition cycle");
    }
}
