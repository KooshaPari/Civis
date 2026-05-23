#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Real-time game state monitoring with live display.
/// </summary>
internal static class WatchCommand
{
    /// <summary>
    /// Creates the <c>watch</c> command.
    /// </summary>
    public static Command Create()
    {
        Command command = new("watch", "Real-time game state monitoring");
        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            using GameClient? client = await CommandHelper.ConnectAsync(ct).ConfigureAwait(false);
            if (client is null) return;

            AnsiConsole.MarkupLine("[dim]Watching game state. Press Ctrl+C to stop.[/]");
            AnsiConsole.WriteLine();

            await AnsiConsole.Live(new Table())
                .StartAsync(async ctx =>
                {
                    while (!ct.IsCancellationRequested)
                    {
                        try
                        {
                            GameStatus status = await client.StatusAsync(ct).ConfigureAwait(false);
                            ResourceSnapshot resources = await client.GetResourcesAsync(ct).ConfigureAwait(false);

                            Table table = new Table()
                                .Border(TableBorder.Rounded)
                                .Title($"[bold]DINOForge Watch[/] [dim]({DateTime.Now:HH:mm:ss})[/]");

                            table.AddColumn("Property");
                            table.AddColumn(new TableColumn("Value").RightAligned());

                            table.AddRow("[bold]-- World --[/]", "");
                            table.AddRow("World Ready", status.WorldReady ? "[green]Yes[/]" : "[yellow]No[/]");
                            table.AddRow("World Name", Markup.Escape(status.WorldName));
                            table.AddRow("Entity Count", status.EntityCount.ToString("N0"));
                            table.AddRow("Loaded Packs", status.LoadedPacks.Count.ToString());

                            table.AddRow("[bold]-- Resources --[/]", "");
                            table.AddRow("Food", resources.Food.ToString("N0"));
                            table.AddRow("Wood", resources.Wood.ToString("N0"));
                            table.AddRow("Stone", resources.Stone.ToString("N0"));
                            table.AddRow("Iron", resources.Iron.ToString("N0"));
                            table.AddRow("Money", resources.Money.ToString("N0"));
                            table.AddRow("Souls", resources.Souls.ToString("N0"));
                            table.AddRow("Bones", resources.Bones.ToString("N0"));
                            table.AddRow("Spirit", resources.Spirit.ToString("N0"));

                            ctx.UpdateTarget(table);
                        }
                        catch (GameClientException)
                        {
                            Table errorTable = new Table()
                                .Border(TableBorder.Rounded)
                                .Title("[red]Connection Lost[/]")
                                .AddColumn("Status")
                                .AddRow("[red]Game disconnected. Retrying...[/]");

                            ctx.UpdateTarget(errorTable);
                        }

                        await Task.Delay(2000, ct).ConfigureAwait(false);
                    }
                }).ConfigureAwait(false);
        });
        return command;
    }
}
