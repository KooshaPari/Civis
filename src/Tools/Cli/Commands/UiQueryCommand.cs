#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Queries UI elements matching a selector.
/// </summary>
internal static class UiQueryCommand
{
    public static Command Create()
    {
        Argument<string> selectorArg = new("selector") { Description = "UI selector (e.g., role=button, text=Mods)" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("ui-query", "Query UI elements by selector");
        command.Add(selectorArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string selector = parseResult.GetRequiredValue(selectorArg);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            UiActionResult result = await client.QueryUiAsync(selector, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            AnsiConsole.MarkupLine($"[bold]Query Result[/]");
            AnsiConsole.MarkupLine($"[dim]Success: {result.Success}[/]");
            AnsiConsole.MarkupLine($"[dim]Match Count: {result.MatchCount}[/]");
            AnsiConsole.MarkupLine($"[dim]Actionable: {result.Actionable}[/]");
            AnsiConsole.MarkupLine($"[dim]Message: {result.Message}[/]");

            if (result.MatchedNode != null)
            {
                RenderNode(result.MatchedNode, 0);
            }
        });
        return command;
    }

    private static void RenderNode(UiNode node, int depth)
    {
        string indent = new string(' ', depth * 2);
        AnsiConsole.MarkupLine($"{indent}[yellow]{node.Name}[/] ({node.Role})");
        if (!string.IsNullOrEmpty(node.Label))
        {
            AnsiConsole.MarkupLine($"{indent}  Label: \"{node.Label}\"");
        }
        AnsiConsole.MarkupLine($"{indent}  Path: {node.Path}");
        AnsiConsole.MarkupLine($"{indent}  Visible: {node.Visible}, Interactable: {node.Interactable}");
    }
}
