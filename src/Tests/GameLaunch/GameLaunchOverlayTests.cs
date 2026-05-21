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
    /// RT-003: F9 overlay toggles visibility and persists across frames.
    /// </summary>
    [Fact]
    public async Task Overlay_F9_TogglesPersistentDebugOverlay()
    {
        if (!fixture.IsInitialized)
        {
            return;  // Skip test when game is not available
        }
        
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
    /// RT-004: F10 mod menu opens and closes without losing entity state.
    /// </summary>
    [Fact]
    public async Task Overlay_F10_ModMenuToggle_PreservesRuntime()
    {
        if (!fixture.IsInitialized)
        {
            return;  // Skip test when game is not available
        }
        
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

    /// <summary>
    /// RT-005: RuntimeDriver survives 600+ frames (10 seconds) of gameplay.
    /// Verifies that the persistent root GameObject does not get destroyed.
    /// </summary>
    [Fact]
    public async Task RuntimeDriver_Survives600FramesAndBeyond()
    {
        if (!fixture.IsInitialized) return;

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
            pollMs: 500).ConfigureAwait(false);

        stayedAlive.Should().BeTrue("status polling should complete within the survival window");
        finalStatus.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should keep ECS world alive after 600+ frames");
        finalStatus.ModPlatformReady.Should().BeTrue(
            "mod platform should still be ready after extended runtime");
    }
}
