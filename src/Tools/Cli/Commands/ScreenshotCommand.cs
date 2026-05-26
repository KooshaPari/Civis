#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Captures a screenshot of the running game.
/// </summary>
internal static class ScreenshotCommand
{
    /// <summary>
    /// Creates the <c>screenshot</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> outputOpt = new("--output") { Description = "Output file path for the screenshot" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("screenshot", "Capture game screenshot");
        command.Add(outputOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string? output = parseResult.GetValue(outputOpt);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            ScreenshotResult result = await client.ScreenshotAsync(output, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]Screenshot saved:[/] {Markup.Escape(result.Path)}");
                AnsiConsole.MarkupLine($"  Size: {result.Width}x{result.Height}");
            }
            else
            {
                AnsiConsole.MarkupLine("[red]Screenshot capture failed.[/]");
            }
        });
        return command;
    }
}
