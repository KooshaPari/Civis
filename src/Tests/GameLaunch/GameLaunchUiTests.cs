#nullable enable
using System.Threading.Tasks;
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
    /// Simulates F10 via the bridge `send_input` tool, then queries `overlay_status`.
    /// Requires the MCP server's overlay_status tool to be implemented in McpServer.
    /// </summary>
    [Fact]
    public async Task Overlay_F10_OpensModMenu()
    {
        // Simulate F10 keypress via bridge input tool
        await fixture.Client!.SendInputAsync("F10");

        // Small delay for Unity OnGUI to process
        await Task.Delay(200);

        DINOForge.Bridge.Protocol.OverlayStatusResult status =
            await fixture.Client.GetOverlayStatusAsync();

        status.Visible.Should().BeTrue(
            "F10 should open the DINOForge mod menu overlay");
    }

    [Fact]
    public async Task Overlay_SecondF10_ClosesModMenu()
    {
        await fixture.Client!.SendInputAsync("F10"); // open
        await Task.Delay(200);
        await fixture.Client.SendInputAsync("F10"); // close
        await Task.Delay(200);

        DINOForge.Bridge.Protocol.OverlayStatusResult status =
            await fixture.Client.GetOverlayStatusAsync();

        status.Visible.Should().BeFalse("second F10 should dismiss the overlay");
    }
}
