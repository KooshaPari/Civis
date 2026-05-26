#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Clicks a UI element by selector.
/// </summary>
internal static class UiClickCommand
{
    public static Command Create()
    {
        Argument<string> selectorArg = new("selector") { Description = "UI selector (e.g., role=button&&text=Mods)" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("ui-click", "Click UI element by selector");
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

            UiActionResult result = await client.ClickUiAsync(selector, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            AnsiConsole.MarkupLine($"[bold]Clicking:[/] {selector}");

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]✓[/] {result.Message}");
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]✗[/] {result.Message}");
                if (!string.IsNullOrEmpty(result.ActionabilityReason))
                {
                    AnsiConsole.MarkupLine($"[red]Reason:[/] {result.ActionabilityReason}");
                }
            }

            if (result.MatchedNode != null)
            {
                AnsiConsole.MarkupLine($"[dim]Target: {result.MatchedNode.Name} ({result.MatchedNode.Role})[/]");
            }
        });
        return command;
    }
}
