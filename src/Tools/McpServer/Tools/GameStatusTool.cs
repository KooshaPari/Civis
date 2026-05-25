#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that queries the current game and mod platform status.
/// </summary>
[McpServerToolType]
public sealed class GameStatusTool
{
    /// <summary>
    /// Returns current game status including world state, entity count, and loaded packs.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="processManager">The process manager (injected).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON status object.</returns>
    [McpServerTool(Name = "game_status"), Description("Get current game status: running state, world readiness, entity count, and loaded packs.")]
    public static async Task<string> GetStatusAsync(
        GameClient client,
        GameProcessManager processManager,
        CancellationToken ct = default)
    {
        bool processRunning = processManager.IsRunning;

        if (!processRunning)
        {
            return GameClientHelper.ToJson(new
            {
                running = false,
                worldReady = false,
                bridgeConnected = false,
                message = "Game is not running."
            });
        }

        if (!await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
        {
            return GameClientHelper.ToJson(new
            {
                running = true,
                worldReady = false,
                bridgeConnected = false,
                message = "Game is running but the DINOForge bridge is not responding."
            });
        }

        try
        {
            GameStatus status = await client.StatusAsync(ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                running = status.Running,
                worldReady = status.WorldReady,
                worldName = status.WorldName,
                entityCount = status.EntityCount,
                modPlatformReady = status.ModPlatformReady,
                loadedPacks = status.LoadedPacks,
                version = status.Version,
                bridgeConnected = true
            });
        }
        catch (GameClientException ex)
        {
            return GameClientHelper.ToJson(new
            {
                running = true,
                bridgeConnected = false,
                error = ex.Message
            });
        }
    }
}
