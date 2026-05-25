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
    public static async Task<string> WaitForWorldAsync(
        GameClient client,
        [Description("Timeout in milliseconds (default: 60000)")] int? timeoutMs = null,
        CancellationToken ct = default)
    {
        if (!await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
        {
            return GameClientHelper.ToJson(new { ready = false, error = GameClientHelper.NotConnectedMessage });
        }

        try
        {
            WaitResult result = await client.WaitForWorldAsync(timeoutMs ?? 60000, ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                ready = result.Ready,
                worldName = result.WorldName
            });
        }
        catch (GameClientException ex)
        {
            return GameClientHelper.ToJson(new { ready = false, error = ex.Message });
        }
    }
}
