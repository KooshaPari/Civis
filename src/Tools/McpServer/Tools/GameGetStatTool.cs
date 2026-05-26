#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that reads stat values from ECS entities via SDK model paths.
/// </summary>
[McpServerToolType]
public sealed class GameGetStatTool
{
    /// <summary>
    /// Reads a stat value from entities by SDK model path.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="sdkPath">Dot-separated SDK model path (e.g. "unit.stats.hp").</param>
    /// <param name="entityIndex">Optional specific entity index to read from.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with stat value, entity count, and component details.</returns>
    [McpServerTool(Name = "game_get-stat"), Description("Read a stat value from ECS entities by SDK model path (e.g. 'unit.stats.hp').")]
    public static Task<string> GetStatAsync(
        GameClient client,
        [Description("Dot-separated SDK model path (e.g. 'unit.stats.hp')")] string sdkPath,
        [Description("Optional specific entity index to read from")] int? entityIndex = null,
        CancellationToken ct = default)
    {
        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.GetStatAsync(sdkPath, entityIndex, token),
            result => new
            {
                sdkPath = result.SdkPath,
                value = result.Value,
                entityCount = result.EntityCount,
                values = result.Values,
                componentType = result.ComponentType,
                fieldName = result.FieldName
            },
            ex => new { sdkPath, error = ex.Message });
    }
}
