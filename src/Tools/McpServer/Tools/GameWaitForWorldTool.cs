#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that blocks until the ECS world is loaded and ready.
/// </summary>
[McpServerToolType]
public sealed class GameWaitForWorldTool
{
    /// <summary>
    /// Waits for the ECS world to become available in the running game.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="timeoutMs">Timeout in milliseconds (default: 60000).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON indicating whether the world is ready.</returns>
    [McpServerTool(Name = "game_wait-for-world"), Description("Block until the ECS world is loaded and ready, or until timeout.")]
    public static Task<string> WaitForWorldAsync(
        GameClient client,
        [Description("Timeout in milliseconds (default: 60000)")] int? timeoutMs = null,
        CancellationToken ct = default)
    {
        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { ready = false, error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.WaitForWorldAsync(timeoutMs ?? 60000, token),
            result => new
            {
                ready = result.Ready,
                worldName = result.WorldName
            },
            ex => new { ready = false, error = ex.Message });
    }
}
