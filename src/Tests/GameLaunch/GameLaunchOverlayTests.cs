#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// RT-003, RT-004, RT-005: F9 debug overlay and F10 mod menu persistence.
/// These tests verify that the overlay system survives ECS frame ticks and
/// responds to input correctly.
/// </summary>
[Collection(GameLaunchCollection.Name)]
[Trait("Category", "GameLaunch")]
public sealed class GameLaunchOverlayTests(GameLaunchFixture fixture)
{
    /// <summary>
    /// RT-003: F9 overlay toggles visibility and persists across frames.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task Overlay_F9_TogglesPersistentDebugOverlay()
    {
        // Send F9 keypress to toggle debug overlay on
        await fixture.Client!.ToggleUiAsync("debugoverlay");
        await Task.Delay(300); // Allow time for UI state change

        // Check entity count is still non-zero (RuntimeDriver alive)
        GameStatus statusAfterToggle = await fixture.Client.StatusAsync();
        statusAfterToggle.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should maintain entity world after F9 toggle");

        // Send F9 again to toggle off
        await fixture.Client.ToggleUiAsync("debugoverlay");
        await Task.Delay(300);

        GameStatus statusAfterSecondToggle = await fixture.Client.StatusAsync();
        statusAfterSecondToggle.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should survive second F9 toggle");
    }

    /// <summary>
    /// RT-004: F10 mod menu opens and closes without losing entity state.
    /// </summary>
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task Overlay_F10_ModMenuToggle_PreservesRuntime()
    {
        // Record initial entity count
        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        int initialEntityCount = initialStatus.EntityCount;
        initialEntityCount.Should().BeGreaterThan(0,
            "ECS world should have entities at baseline");

        // Open mod menu via F10
        await fixture.Client.ToggleUiAsync("modmenu");
        await Task.Delay(300);

        GameStatus openStatus = await fixture.Client.StatusAsync();
        openStatus.EntityCount.Should().Be(initialEntityCount,
            "entity count should remain unchanged when mod menu opens");

        // Close mod menu
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
    [Fact(Skip = "Requires live game with DINOForge loaded")]
    public async Task RuntimeDriver_Survives600FramesAndBeyond()
    {
        GameStatus initialStatus = await fixture.Client!.StatusAsync();
        initialStatus.EntityCount.Should().BeGreaterThan(0,
            "ECS world should be populated at start");

        // Wait 10 seconds (600 frames at 60 FPS)
        await Task.Delay(10_000);

        // Check that entities are still present
        GameStatus finalStatus = await fixture.Client.StatusAsync();
        finalStatus.EntityCount.Should().BeGreaterThan(0,
            "RuntimeDriver should keep ECS world alive after 600+ frames");

        // Verify mod platform is still responsive
        finalStatus.ModPlatformReady.Should().BeTrue(
            "mod platform should still be ready after extended runtime");
    }
}
