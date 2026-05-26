#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Applies a stat override to entities in the game.
/// </summary>
internal static class OverrideCommand
{
    /// <summary>
    /// Creates the <c>override</c> command.
    /// </summary>
    public static Command Create()
    {
        Argument<string> pathArg = new("path") { Description = "SDK path to override (e.g. unit.stats.hp)" };
        Argument<float> valueArg = new("value") { Description = "Value to apply" };
        Option<string?> modeOpt = new("--mode") { Description = "Override mode: override, add, or multiply" };
        Option<string?> filterOpt = new("--filter") { Description = "Entity filter expression" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        Command command = new("override", "Apply stat override");
        command.Add(pathArg);
        command.Add(valueArg);
        command.Add(modeOpt);
        command.Add(filterOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string path = parseResult.GetRequiredValue(pathArg);
            float value = parseResult.GetRequiredValue(valueArg);
            string? mode = parseResult.GetValue(modeOpt);
            string? filter = parseResult.GetValue(filterOpt);

            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            OverrideResult result = await client.ApplyOverrideAsync(path, value, mode, filter, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            if (result.Success)
            {
                AnsiConsole.MarkupLine($"[green]Override applied:[/] {Markup.Escape(result.SdkPath)}");
                AnsiConsole.MarkupLine($"  Modified: [bold]{result.ModifiedCount}[/] entities");
                AnsiConsole.MarkupLine($"  {Markup.Escape(result.Message)}");
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]Override failed:[/] {Markup.Escape(result.Message)}");
            }
        });
        return command;
    }
}
