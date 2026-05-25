#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that returns SDK-to-ECS component type mappings.
/// </summary>
[McpServerToolType]
public sealed class GameGetComponentMapTool
{
    /// <summary>
    /// Returns the component mapping table linking SDK model paths to ECS component types.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="sdkPath">Optional SDK path filter. If omitted, returns all mappings.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with component mapping entries.</returns>
    [McpServerTool(Name = "game_get-component-map"), Description("Get SDK-to-ECS component type mappings. Optionally filter by SDK path.")]
    public static async Task<string> GetComponentMapAsync(
        GameClient client,
        [Description("Optional SDK path filter; omit to return all mappings")] string? sdkPath = null,
        CancellationToken ct = default)
    {
        if (!await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
        {
            return GameClientHelper.ToJson(new { error = GameClientHelper.NotConnectedMessage });
        }

        try
        {
            ComponentMapResult result = await client.GetComponentMapAsync(sdkPath, ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                mappings = result.Mappings.Select(m => new
                {
                    sdkPath = m.SdkPath,
                    ecsType = m.EcsType,
                    fieldName = m.FieldName,
                    resolved = m.Resolved,
                    description = m.Description
                })
            });
        }
        catch (GameClientException ex)
        {
            return GameClientHelper.ToJson(new { error = ex.Message });
        }
    }
}
