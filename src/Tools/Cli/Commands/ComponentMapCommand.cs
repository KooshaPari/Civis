#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Shows the SDK-to-ECS component mapping table.
/// </summary>
internal static class ComponentMapCommand
{
    /// <summary>
    /// Creates the <c>component-map</c> command.
    /// </summary>
    public static Command Create()
    {
        Option<string?> pathOpt = new("--path") { Description = "Filter by SDK path prefix" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("component-map", "Show component mappings");
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

            ComponentMapResult result = await client.GetComponentMapAsync(path, ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(result);
                return;
            }

            if (result.Mappings.Count == 0)
            {
                AnsiConsole.MarkupLine("[dim]No component mappings found.[/]");
                return;
            }

            Table table = new Table()
                .Border(TableBorder.Rounded)
                .Title("[bold]Component Mappings[/]")
                .AddColumn("SDK Path")
                .AddColumn("ECS Type")
                .AddColumn("Field")
                .AddColumn("Resolved");

            foreach (ComponentMapEntry entry in result.Mappings)
            {
                table.AddRow(
                    Markup.Escape(entry.SdkPath),
                    Markup.Escape(entry.EcsType),
                    Markup.Escape(entry.FieldName),
                    entry.Resolved ? "[green]Yes[/]" : "[red]No[/]");
            }

            AnsiConsole.Write(table);
        });
        return command;
    }
}
