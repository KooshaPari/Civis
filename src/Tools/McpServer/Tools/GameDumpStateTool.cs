#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that dumps the full ECS state for a given category.
/// </summary>
[McpServerToolType]
public sealed class GameDumpStateTool
{
    /// <summary>
    /// Dumps the ECS game state, optionally filtered by category.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="category">Category to dump: "unit", "building", "projectile", or "all" (default).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with catalog snapshot organized by category.</returns>
    [McpServerTool(Name = "game_dump-state"), Description("Dump ECS game state. Optionally filter by category: unit, building, projectile, or all.")]
    public static Task<string> DumpStateAsync(
        GameClient client,
        [Description("Category to dump: 'unit', 'building', 'projectile', or 'all' (default)")] string? category = null,
        CancellationToken ct = default)
    {
        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.DumpStateAsync(category, token),
            result => new
            {
                units = result.Units.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                buildings = result.Buildings.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                projectiles = result.Projectiles.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category }),
                other = result.Other.Select(e => new { e.InferredId, e.ComponentCount, e.EntityCount, e.Category })
            });
    }
}
