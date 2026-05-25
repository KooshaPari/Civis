#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that triggers a content pack reload from disk.
/// </summary>
[McpServerToolType]
public sealed class GameReloadPacksTool
{
    /// <summary>
    /// Reloads content packs from the packs directory.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="path">Optional packs directory path override.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with loaded packs and any errors.</returns>
    [McpServerTool(Name = "game_reload-packs"), Description("Reload content packs from disk. Returns loaded pack IDs and any errors.")]
    public static Task<string> ReloadPacksAsync(
        GameClient client,
        [Description("Optional packs directory path override")] string? path = null,
        CancellationToken ct = default)
    {
        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { success = false, error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.ReloadPacksAsync(path, token),
            result => new
            {
                success = result.Success,
                loadedPacks = result.LoadedPacks,
                errors = result.Errors
            },
            ex => new { success = false, error = ex.Message });
    }
}
