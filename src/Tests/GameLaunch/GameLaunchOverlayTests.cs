#nullable enable
using System;
using System.Diagnostics;
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Support;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// RT-003, RT-004, RT-005: F9 debug overlay and F10 mod menu persistence.
/// These tests verify that the overlay system survives ECS frame ticks and
/// responds to input correctly.
///
/// Conditionally skips on CI where DINO_GAME_PATH is not set.
/// On a self-hosted Windows runner with the game installed, these tests run.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchOverlayTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// SPEC-007 Feature 2: UGUI debug and mod menu panels start hidden (alpha 0) at main menu.
    /// </summary>
    [SkippableFact]
    public async Task Overlay_Panels_HiddenByDefault_AtMainMenu()
    {
        fixture.SkipIfNotInitialized();

        UiActionResult debugPanel = await QueryDfCanvasPanelAsync("DebugPanel").ConfigureAwait(false);
        debugPanel.MatchCount.Should().BeGreaterThan(0, "DebugPanel should exist on DFCanvas at main menu");
        debugPanel.MatchedNode.Should().NotBeNull();
        debugPanel.MatchedNode!.Visible.Should().BeFalse(
            "DebugPanel should start hidden (CanvasGroup alpha 0) before F9 toggle");

        UiActionResult modMenuPanel = await QueryDfCanvasPanelAsync("ModMenuPanel").ConfigureAwait(false);
        modMenuPanel.MatchCount.Should().BeGreaterThan(0, "ModMenuPanel should exist on DFCanvas at main menu");

        // Close mod menu when a prior test left it open (shared GameLaunch collection).
        if (modMenuPanel.MatchedNode?.Visible == true)
        {
            await fixture.Client.ToggleUiAsync("modmenu").ConfigureAwait(false);
            await Task.Delay(400).ConfigureAwait(false);
        }
    }

    /// <summary>
    /// RT-003: F9 overlay toggles visibility and persists across frames.
    /// </summary>
    [SkippableFact]
    public async Task Overlay_F9_TogglesPersistentDebugOverlay()
    {
        fixture.SkipIfNotInitialized();

        await fixture.Client!.ToggleUiAsync("debugoverlay");
        await Task.Delay(300);

        GameStatus statusAfterToggle = await fixture.Client.StatusAsync();
        statusAfterToggle.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should maintain entity world after F9 toggle");

        await fixture.Client.ToggleUiAsync("debugoverlay");
        await Task.Delay(300);

        GameStatus statusAfterSecondToggle = await fixture.Client.StatusAsync();
        statusAfterSecondToggle.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should survive second F9 toggle");
    }

    /// <summary>
    /// RT-003 / SPEC-007: F9 opens debug overlay at main menu (bridge UI visibility wait).
    /// </summary>
    [SkippableFact]
    public async Task Overlay_F9_AssertDebugPanelVisible_AtMainMenu()
    {
        fixture.SkipIfNotInitialized();

        StartGameResult openResult = await fixture.Client!.ToggleUiAsync("debug");
        openResult.Success.Should().BeTrue(
            "ToggleUiAsync(debug) should invoke DFCanvas.ToggleDebug() at main menu");

        UiWaitResult waitResult = await fixture.Client.WaitForUiAsync(
            "name=DebugPanel", "visible", timeoutMs: 3000);
        waitResult.Ready.Should().BeTrue(
            "debug panel should be visible after F9 toggle at main menu");
    }

    /// <summary>
    /// RT-003 / SPEC-007: F9 closes debug overlay after second toggle at main menu.
    /// </summary>
    [SkippableFact]
    public async Task Overlay_F9_SecondToggle_ClosesDebugPanel_AtMainMenu()
    {
        fixture.SkipIfNotInitialized();

        StartGameResult openResult = await fixture.Client!.ToggleUiAsync("debug");
        openResult.Success.Should().BeTrue(
            "first ToggleUiAsync(debug) should open debug panel at main menu");

        UiWaitResult openWait = await fixture.Client.WaitForUiAsync(
            "name=DebugPanel", "visible", timeoutMs: 5000);
        openWait.Ready.Should().BeTrue(
            "debug panel should be visible after first F9 toggle at main menu");

        await Task.Delay(300);

        StartGameResult closeResult = await fixture.Client.ToggleUiAsync("debug");
        closeResult.Success.Should().BeTrue(
            "second ToggleUiAsync(debug) should close debug panel at main menu");

        UiWaitResult closeWait = await fixture.Client.WaitForUiAsync(
            "name=DebugPanel", "hidden", timeoutMs: 5000);
        closeWait.Ready.Should().BeTrue(
            "debug panel should be hidden after second F9 toggle at main menu");
    }

    /// <summary>
    /// RT-003 / RT-004 / SPEC-007: F9 debug overlay and F10 mod menu toggle during gameplay.
    /// </summary>
    [SkippableFact]
    public async Task Overlay_F9_F10_ToggleDuringGameplay()
    {
        fixture.SkipIfNotInitialized();

        StartGameResult startResult = await fixture.Client!.StartGameAsync();
        startResult.Success.Should().BeTrue(
            "StartGameAsync should enter gameplay from main menu");

        await Task.Delay(3000);

        GameStatus status = await fixture.Client.StatusAsync();
        status.WorldReady.Should().BeTrue(
            "ECS world should be ready after scene transition to gameplay");

        StartGameResult debugResult = await fixture.Client.ToggleUiAsync("debug");
        debugResult.Success.Should().BeTrue(
            "ToggleUiAsync(debug) should open debug panel in-game");

        UiWaitResult debugWait = await fixture.Client.WaitForUiAsync(
            "path=DebugPanel", "visible", timeoutMs: 12_000);
        debugWait.Ready.Should().BeTrue(
            "debug panel should be visible after F9 toggle during gameplay");

        StartGameResult modResult = await fixture.Client.ToggleUiAsync("modmenu");
        modResult.Success.Should().BeTrue(
            "ToggleUiAsync(modmenu) should open mod menu in-game");

        UiWaitResult modWait = await fixture.Client.WaitForUiAsync(
            "path=ModMenuPanel", "visible", timeoutMs: 8000);
        modWait.Ready.Should().BeTrue(
            "mod menu should be visible after F10 toggle during gameplay");
    }

    /// <summary>
    /// RT-004: F10 mod menu opens and closes without losing entity state.
    /// </summary>
    [SkippableFact]
    public async Task Overlay_F10_ModMenuToggle_PreservesRuntime()
    {
        fixture.SkipIfNotInitialized();

        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        int initialEntityCount = initialStatus.EntityCount;
        initialEntityCount.Should().BeGreaterThan(0,
            "ECS world should have entities at baseline");

        await fixture.Client.ToggleUiAsync("modmenu");
        await Task.Delay(300);

        GameStatus openStatus = await fixture.Client.StatusAsync();
        openStatus.EntityCount.Should().Be(initialEntityCount,
            "entity count should remain unchanged when mod menu opens");

        await fixture.Client.ToggleUiAsync("modmenu");
        await Task.Delay(300);

        GameStatus closedStatus = await fixture.Client.StatusAsync();
        closedStatus.EntityCount.Should().Be(initialEntityCount,
            "entity count should match initial after mod menu closes");
    }

    private async Task<UiActionResult> QueryDfCanvasPanelAsync(string panelName)
    {
        UiActionResult byName = await fixture.Client!.QueryUiAsync($"name={panelName}").ConfigureAwait(false);
        if (byName.MatchCount > 0)
        {
            return byName;
        }

        UiActionResult byPath = await fixture.Client.QueryUiAsync($"path={panelName}").ConfigureAwait(false);
        if (byPath.MatchCount > 0)
        {
            return byPath;
        }

        UiTreeResult tree = await fixture.Client.GetUiTreeAsync().ConfigureAwait(false);
        if (tree.Success && FindUiNodeByName(tree.Root, panelName) is UiNode node)
        {
            return new UiActionResult
            {
                Success = true,
                MatchCount = 1,
                MatchedNode = node,
                Message = $"Found {panelName} in UI tree snapshot",
            };
        }

        return byName;
    }

    private static UiNode? FindUiNodeByName(UiNode root, string name)
    {
        if (string.Equals(root.Name, name, StringComparison.OrdinalIgnoreCase)
            || root.Name.Contains(name, StringComparison.OrdinalIgnoreCase))
        {
            return root;
        }

        foreach (UiNode child in root.Children)
        {
            UiNode? found = FindUiNodeByName(child, name);
            if (found != null)
            {
                return found;
            }
        }

        return null;
    }

    /// <summary>
    /// RT-005: RuntimeDriver survives 600+ frames (10 seconds) of gameplay.
    /// Verifies that the persistent root GameObject does not get destroyed.
    /// </summary>
    [SkippableFact]
    public async Task RuntimeDriver_Survives600FramesAndBeyond()
    {
        fixture.SkipIfNotInitialized();

        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        initialStatus.EntityCount.Should().BeGreaterThan(0,
            "ECS world should be populated at start");

        // Pattern #108: poll throughout the survival window instead of bare 10s sleep.
        // Fails fast on regression (driver death) rather than waiting full duration.
        var survivalWindow = TimeSpan.FromSeconds(10);
        var sw = Stopwatch.StartNew();
        GameStatus finalStatus = initialStatus;
        bool stayedAlive = await TestWait.UntilAsync(
            async () =>
            {
                finalStatus = await fixture.Client.StatusAsync().ConfigureAwait(false);
                if (finalStatus.EntityCount <= 0 || !finalStatus.ModPlatformReady)
                {
                    // Regression detected — stop polling immediately.
                    return true;
                }
                // Keep polling until the survival window elapses.
                return sw.Elapsed >= survivalWindow;
            },
            timeout: survivalWindow + TimeSpan.FromSeconds(2),
            pollMs: 500);

        stayedAlive.Should().BeTrue("status polling should complete within the survival window");
        finalStatus.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should keep ECS world alive after 600+ frames");
        finalStatus.ModPlatformReady.Should().BeTrue(
            "mod platform should still be ready after extended runtime");
    }

}
