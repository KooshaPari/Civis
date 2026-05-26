#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Tools.McpServer;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that performs end-to-end mod verification: loads a pack,
/// verifies entity changes, and reports results.
/// </summary>
[McpServerToolType]
public sealed class GameVerifyModTool
{
    /// <summary>
    /// Loads a content pack into the running game and verifies that it applies correctly.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="packPath">Path to the pack directory or manifest file.</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with verification results including stat changes and errors.</returns>
    [McpServerTool(Name = "game_verify-mod"), Description("End-to-end mod verification: load a pack, verify entity changes, and report results.")]
    public static Task<string> VerifyModAsync(
        GameClient client,
        [Description("Path to the pack directory or manifest file")] string packPath,
        CancellationToken ct = default)
    {
        string safePackPath;
        try
        {
            safePackPath = McpPathSafety.ResolvePackPath(packPath);
        }
        catch (Exception ex)
        {
            return Task.FromResult(GameClientHelper.ToJson(new { loaded = false, packPath, error = ex.Message }));
        }

        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { loaded = false, error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.VerifyModAsync(safePackPath, token),
            result => new
            {
                packId = result.PackId,
                loaded = result.Loaded,
                statChanges = result.StatChanges,
                errors = result.Errors,
                entityCount = result.EntityCount
            },
            ex => new { loaded = false, packPath, error = ex.Message });
    }
}
