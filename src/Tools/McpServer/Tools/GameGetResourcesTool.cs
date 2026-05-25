#nullable enable
using System.ComponentModel;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using ModelContextProtocol.Server;

namespace DINOForge.Tools.McpServer.Tools;

/// <summary>
/// MCP tool that reads current in-game resource stockpile values.
/// </summary>
[McpServerToolType]
public sealed class GameGetResourcesTool
{
    /// <summary>
    /// Returns all current resource values from the running game.
    /// </summary>
    /// <param name="client">The game client (injected).</param>
    /// <param name="ct">Cancellation token.</param>
    /// <returns>JSON with all resource stockpile values.</returns>
    [McpServerTool(Name = "game_get-resources"), Description("Get all current in-game resource stockpile values (food, wood, stone, iron, money, souls, bones, spirit).")]
    public static Task<string> GetResourcesAsync(
        GameClient client,
        CancellationToken ct = default)
    {
        return GameClientHelper.InvokeBridgeAsync(
            client,
            ct,
            new { error = GameClientHelper.NotConnectedMessage },
            (c, token) => c.GetResourcesAsync(token),
            result => new
            {
                food = result.Food,
                wood = result.Wood,
                stone = result.Stone,
                iron = result.Iron,
                money = result.Money,
                souls = result.Souls,
                bones = result.Bones,
                spirit = result.Spirit
            });
    }
}
