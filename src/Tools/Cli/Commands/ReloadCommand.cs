#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Reloads content packs in the running game.
/// </summary>
internal static class ReloadCommand
{
    /// <summary>
    /// Creates the <c>reload</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> pathOpt = new("--path") { Description = "Path to packs directory to reload from" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("reload", "Reload packs");
        command.Add(pathOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string? path = parseResult.GetValue(pathOpt);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            ReloadResult result = await client.ReloadPacksAsync(path, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            if (result.Success)
            {
                AnsiConsole.MarkupLine("[green]Reload successful.[/]");
            }
            else
            {
                AnsiConsole.MarkupLine("[red]Reload completed with errors.[/]");
            }

            if (result.LoadedPacks.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[bold]Loaded Packs:[/]");
                foreach (string pack in result.LoadedPacks)
                {
                    AnsiConsole.MarkupLine($"  [green]+[/] {Markup.Escape(pack)}");
                }
            }
            else
            {
                AnsiConsole.MarkupLine("[dim]No packs loaded.[/]");
            }

            if (result.Errors.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[bold red]Errors:[/]");
                foreach (string error in result.Errors)
                {
                    AnsiConsole.MarkupLine($"  [red]x[/] {Markup.Escape(error)}");
                }
            }
        });
        return command;
    }
}
