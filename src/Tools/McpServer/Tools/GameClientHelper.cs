#nullable enable
using System.Text.Json;
using DINOForge.Bridge.Client;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// Shared helper for MCP tool implementations. Provides safe connection
/// handling and JSON serialization for game bridge results.
/// </summary>
internal static class GameClientHelper
{
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        WriteIndented = true,
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase
    };

    /// <summary>
    /// Ensures the <see cref="GameClient"/> is connected to the game bridge.
    /// If not connected, attempts to connect with a short timeout.
    /// </summary>
    /// <param name="client">The game client instance.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>True if connected; false if the game is unreachable.</returns>
    internal static async Task<bool> EnsureConnectedAsync(GameClient client, CancellationToken ct = default)
    {
        if (client.IsConnected)
            return true;

        try
        {
            await client.ConnectAsync(ct).ConfigureAwait(false);
            return true;
        }
        catch
        {
            return false;
        }
    }

    /// <summary>
    /// Serializes a result object to a formatted JSON string for MCP tool output.
    /// </summary>
    /// <typeparam name="T">The result type.</typeparam>
    /// <param name="result">The result to serialize.</param>
    /// <returns>A formatted JSON string.</returns>
    internal static string ToJson<T>(T result)
    {
        return JsonSerializer.Serialize(result, JsonOptions);
    }

    /// <summary>
    /// Returns a standard error message when the game is not running or unreachable.
    /// </summary>
    internal static string NotConnectedMessage =>
        "Game is not running or the DINOForge bridge is unreachable. " +
        "Use 'game_launch' to start the game, or ensure the DINOForge Runtime plugin is loaded.";
}
