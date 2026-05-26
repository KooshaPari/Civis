#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Asserts UI state (visible, text, count, etc.).
/// </summary>
internal static class UiExpectCommand
{
    public static Command Create()
    {
        Argument<string> selectorArg = new("selector") { Description = "UI selector" };
        Argument<string> conditionArg = new("condition") { Description = "Condition: visible, hidden, interactable, text=..., text-exact=..., count=N, count>=N" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("ui-expect", "Assert UI element state");
        command.Add(selectorArg);
        command.Add(conditionArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string selector = parseResult.GetRequiredValue(selectorArg);
            string condition = parseResult.GetRequiredValue(conditionArg);

            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            UiExpectationResult result = await client.ExpectUiAsync(selector, condition, ct).ConfigureAwait(false);

            if (json)
            {
                if (!result.Success)
                {
                    Environment.ExitCode = 1;
                }

                CommandOutput.WriteJson(result);
                return;
            }

            AnsiConsole.MarkupLine($"[bold]Expecting:[/] {selector} -> {condition}");

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]✓ PASS[/] {result.Message}");
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]✗ FAIL[/] {result.Message}");
            }

            AnsiConsole.MarkupLine($"[dim]Match count: {result.MatchCount}[/]");

            // Exit with error code on failure for CI/scripting
            if (!result.Success)
            {
                Environment.Exit(1);
            }
        });
        return command;
    }
}
