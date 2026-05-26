#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Waits for UI state (visible, hidden, interactable).
/// </summary>
internal static class UiWaitCommand
{
    public static Command Create()
    {
        Argument<string> selectorArg = new("selector") { Description = "UI selector" };
        Option<string> stateOpt = new("--state") { Description = "State to wait for: visible, hidden, interactable, actionable" };
        Option<int> timeoutOpt = new("--timeout") { Description = "Timeout in milliseconds" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("ui-wait", "Wait for UI element state");
        command.Add(selectorArg);
        command.Add(stateOpt);
        command.Add(timeoutOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string selector = parseResult.GetRequiredValue(selectorArg);
            string? state = parseResult.GetValue(stateOpt) ?? "visible";
            int timeout = parseResult.GetValue(timeoutOpt);
            if (timeout == 0) timeout = 5000;

            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            UiWaitResult result = await client.WaitForUiAsync(selector, state, timeout, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            AnsiConsole.MarkupLine($"[bold]Waiting:[/] {selector} -> {state} (timeout: {timeout}ms)");

            if (result.Ready)
            {
                AnsiConsole.MarkupLine($"[green]✓[/] {result.Message}");
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]✗[/] {result.Message}");
            }

            AnsiConsole.MarkupLine($"[dim]Match count: {result.MatchCount}[/]");
        });
        return command;
    }
}
