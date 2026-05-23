#nullable enable
using System.CommandLine;
using DINOForge.Bridge.Client;
using DINOForge.Bridge.Protocol;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Dumps game state (units, buildings, projectiles, or all).
/// </summary>
internal static class DumpCommand
{
    /// <summary>
    /// Creates the <c>dump</c> command.
    /// </summary>
    public static Command Create()
    {
        Argument<string?> categoryArg = new("category")
        {
            Description = "Category to dump: units, buildings, projectiles, or all",
            DefaultValueFactory = _ => null
        };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();
        Command command = new("dump", "Dump game state (units/buildings/projectiles/all)");
        command.Add(categoryArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            bool json = CommandOutput.IsJson(parseResult, formatOpt);
            string? category = parseResult.GetValue(categoryArg);
            using GameClient? client = await CommandHelper.ConnectAsync(ct, writeErrors: !json).ConfigureAwait(false);
            if (client is null)
            {
                if (json)
                {
                    CommandOutput.WriteJsonError("game_not_running", "Game not running. Start DINO first.");
                }

                return;
            }

            CatalogSnapshot catalog = category is not null
                ? await client.DumpStateAsync(category, ct).ConfigureAwait(false)
                : await client.GetCatalogAsync(ct).ConfigureAwait(false);

            if (json)
            {
                CommandOutput.WriteJson(catalog);
                return;
            }

            RenderCategory("Units", catalog.Units);
            RenderCategory("Buildings", catalog.Buildings);
            RenderCategory("Projectiles", catalog.Projectiles);
            RenderCategory("Other", catalog.Other);
        });
        return command;
    }

    private static void RenderCategory(string title, List<CatalogEntry> entries)
    {
        if (entries.Count == 0) return;

        Table table = new Table()
            .Border(TableBorder.Rounded)
            .Title($"[bold]{Markup.Escape(title)}[/] ({entries.Count})")
            .AddColumn("ID")
            .AddColumn("Components")
            .AddColumn("Entities");

        foreach (CatalogEntry entry in entries)
        {
            table.AddRow(
                Markup.Escape(entry.InferredId),
                entry.ComponentCount.ToString(),
                entry.EntityCount.ToString("N0"));
        }

        AnsiConsole.Write(table);
        AnsiConsole.WriteLine();
    }
}
