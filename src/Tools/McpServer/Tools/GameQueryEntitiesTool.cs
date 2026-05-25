#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that queries entities in the ECS world by component type or category.
/// </summary>
[McpServerToolType]
public sealed class GameQueryEntitiesTool
{
    /// <summary>
    /// Queries entities matching the specified component type and/or category filter.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="componentType">ECS component type name to filter by (e.g. "Components.Health").</param>
    /// <param name="category">Category filter: "unit", "building", "projectile", or "other".</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with entity count and details.</returns>
    [McpServerTool(Name = "game_query-entities"), Description("Query entities in the ECS world by component type and/or category.")]
    public static async Task<string> QueryEntitiesAsync(
        GameClient client,
        [Description("ECS component type name to filter by (e.g. 'Components.Health')")] string? componentType = null,
        [Description("Category filter: unit, building, projectile, or other")] string? category = null,
        CancellationToken ct = default)
    {
        if (!await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
        {
            return GameClientHelper.ToJson(new { count = 0, error = GameClientHelper.NotConnectedMessage });
        }

        try
        {
            QueryResult result = await client.QueryEntitiesAsync(componentType, category, ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                count = result.Count,
                entities = result.Entities.Select(e => new
                {
                    index = e.Index,
                    components = e.Components
                })
            });
        }
        catch (GameClientException ex)
        {
            return GameClientHelper.ToJson(new { count = 0, error = ex.Message });
        }
    }
}
