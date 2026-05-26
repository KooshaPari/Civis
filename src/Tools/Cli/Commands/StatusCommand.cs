#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Shows game and mod platform status.
/// </summary>
internal static class StatusCommand
{
    /// <summary>
    /// Creates the <c>status</c> command.
    /// </summary>
    public static Command Create()
    {
        Command command = new("status", "Show game and mod platform status");
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        command.Add(formatOpt);
        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            Bridge.Protocol.GameStatus status = await client.StatusAsync(ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(status);
                return;
            }

            Table table = new Table()
                .Border(TableBorder.Rounded)
                .Title("[bold]DINOForge Status[/]")
                .AddColumn("Property")
                .AddColumn("Value");

            table.AddRow("Running", status.Running ? "[green]Yes[/]" : "[red]No[/]");
            table.AddRow("World Ready", status.WorldReady ? "[green]Yes[/]" : "[yellow]No[/]");
            table.AddRow("World Name", Markup.Escape(status.WorldName));
            table.AddRow("Entity Count", status.EntityCount.ToString("N0"));
            table.AddRow("Mod Platform Ready", status.ModPlatformReady ? "[green]Yes[/]" : "[yellow]No[/]");
            table.AddRow("Version", Markup.Escape(status.Version));
            table.AddRow("Loaded Packs", status.LoadedPacks.Count > 0
                ? Markup.Escape(string.Join(", ", status.LoadedPacks))
                : "[dim]None[/]");

            AnsiConsole.Write(table);
        });
        return command;
    }
}
