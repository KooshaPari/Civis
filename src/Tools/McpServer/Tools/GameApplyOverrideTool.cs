#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that applies stat overrides to matching ECS entities.
/// </summary>
[McpServerToolType]
public sealed class GameApplyOverrideTool
{
    /// <summary>
    /// Applies a stat override to matching entities in the ECS world.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="sdkPath">SDK model path of the stat to modify (e.g. "unit.stats.hp").</param>
    /// <param name="value">The value to apply.</param>
    /// <param name="mode">Application mode: "override" (default), "add", or "multiply".</param>
    /// <param name="filter">Optional filter component type name to narrow affected entities.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with success status and modified entity count.</returns>
    [McpServerTool(Name = "game_apply-override"), Description("Apply a stat override to matching ECS entities. Supports override, add, and multiply modes.")]
    public static async Task<string> ApplyOverrideAsync(
        GameClient client,
        [Description("SDK model path of the stat to modify (e.g. 'unit.stats.hp')")] string sdkPath,
        [Description("The value to apply")] float value,
        [Description("Application mode: 'override' (default), 'add', or 'multiply'")] string? mode = null,
        [Description("Optional filter component type name to narrow affected entities")] string? filter = null,
        CancellationToken ct = default)
    {
        if (!await GameClientHelper.EnsureConnectedAsync(client, ct).ConfigureAwait(false))
        {
            return GameClientHelper.ToJson(new { success = false, error = GameClientHelper.NotConnectedMessage });
        }

        try
        {
            OverrideResult result = await client.ApplyOverrideAsync(sdkPath, value, mode, filter, ct).ConfigureAwait(false);
            return GameClientHelper.ToJson(new
            {
                success = result.Success,
                modifiedCount = result.ModifiedCount,
                sdkPath = result.SdkPath,
                message = result.Message
            });
        }
        catch (GameClientException ex)
        {
            return GameClientHelper.ToJson(new { success = false, sdkPath, error = ex.Message });
        }
    }
}
