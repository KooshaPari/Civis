#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// End-to-end pack verification against the running game.
/// </summary>
internal static class VerifyPackCommand
{
    /// <summary>
    /// Creates the <c>verify</c> command.
    /// </summary>
    public static Command Create()
    {
        Argument<string> packPathArg = new("packPath") { Description = "Path to the pack directory to verify" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("verify", "End-to-end pack verification");
        command.Add(packPathArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string packPath = parseResult.GetRequiredValue(packPathArg);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            VerifyResult result = await client.VerifyModAsync(packPath, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            Table table = new Table()
                .Border(TableBorder.Rounded)
                .Title("[bold]Pack Verification Report[/]")
                .AddColumn("Property")
                .AddColumn("Value");

            table.AddRow("Pack ID", Markup.Escape(result.PackId));
            table.AddRow("Loaded", result.Loaded ? "[green]Yes[/]" : "[red]No[/]");
            table.AddRow("Entity Count", result.EntityCount.ToString("N0"));

            AnsiConsole.Write(table);

            if (result.StatChanges.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[bold]Stat Changes:[/]");
                foreach (string change in result.StatChanges)
                {
                    AnsiConsole.MarkupLine($"  [green]+[/] {Markup.Escape(change)}");
                }
            }

            if (result.Errors.Count > 0)
            {
                AnsiConsole.MarkupLine("\n[bold red]Errors:[/]");
                foreach (string error in result.Errors)
                {
                    AnsiConsole.MarkupLine($"  [red]x[/] {Markup.Escape(error)}");
                }
            }

            if (result.Errors.Count == 0 && result.Loaded)
            {
                AnsiConsole.MarkupLine("\n[green bold]PASS[/]");
            }
            else
            {
                AnsiConsole.MarkupLine("\n[red bold]FAIL[/]");
            }
        });
        return command;
    }
}
