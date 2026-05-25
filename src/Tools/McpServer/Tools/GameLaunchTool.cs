#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that launches the Diplomacy is Not an Option game process
/// and waits for the DINOForge bridge pipe to become available.
/// </summary>
[McpServerToolType]
public sealed class GameLaunchTool
{
    /// <summary>
    /// Launches the game and connects to the DINOForge bridge.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="processManager">The process manager (injected).</param>
    /// <param name="gamePath">Optional path to the game executable. If omitted, attempts Steam launch then probes common install locations.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>Launch status and connection result.</returns>
    [McpServerTool(Name = "game_launch"), Description("Launch Diplomacy is Not an Option and connect to the DINOForge bridge.")]
    public static async Task<string> LaunchAsync(
        GameClient client,
        GameProcessManager processManager,
        [Description("Optional path to the game executable")] string? gamePath = null,
        CancellationToken ct = default)
    {
        if (client.IsConnected)
        {
            return GameClientHelper.ToJson(new { success = true, message = "Game is already running and bridge is connected." });
        }

        if (processManager.IsRunning)
        {
            // Game is running but not connected - try connecting
            bool connected = await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                success = connected,
                message = connected
                    ? "Game was already running. Bridge connected successfully."
                    : "Game is running but the DINOForge bridge is not responding. Ensure the Runtime plugin is loaded."
            });
        }

        bool launched = await processManager.LaunchAsync(gamePath).ConfigureAwait(false);
        if (!launched)
        {
            return GameClientHelper.ToJson(new
            {
                success = false,
                message = "Failed to launch the game. Provide a gamePath or ensure Steam is installed."
            });
        }

        // Wait for bridge to become available (game needs time to load)
        for (int attempt = 0; attempt < 12; attempt++)
        {
            await Task.Delay(5000, ct).ConfigureAwait(false);
            if (await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
            {
                return GameClientHelper.ToJson(new { success = true, message = "Game launched and bridge connected." });
            }
        }

        return GameClientHelper.ToJson(new
        {
            success = false,
            message = "Game launched but the DINOForge bridge did not respond within 60 seconds. The Runtime plugin may not be loaded."
        });
    }
}
