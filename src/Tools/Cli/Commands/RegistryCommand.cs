#nullable enable
using System.CommandLine;
using System.Net.Http;
using System.Text.Json;
using DINOForge.SDK.Json;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Pack registry commands (list, search, info).
/// </summary>
internal static class RegistryCommand
{
    /// <summary>
    /// Creates the <c>registry</c> command group.
    /// </summary>
    public static Command Create()
    {
        Command command = new("registry", "Browse and search the pack registry");

        command.Add(CreateListCommand());
        command.Add(CreateSearchCommand());
        command.Add(CreateInfoCommand());

        return command;
    }

    /// <summary>
    /// Creates the <c>registry list</c> subcommand.
    /// </summary>
    private static Command CreateListCommand()
    {
        Command command = new("list", "List all available packs");

        Option<string?> typeOpt = new("--type", "-t") { Description = "Filter by pack type (content, balance, scenario, total_conversion, utility, ruleset)" };

        Option<string?> searchOpt = new("--search", "-s") { Description = "Search by name or description" };

        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        command.Add(typeOpt);
        command.Add(searchOpt);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string? type = parseResult.GetValue(typeOpt);
            string? search = parseResult.GetValue(searchOpt);
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var registry = await FetchRegistryAsync(ct).ConfigureAwait(false);

                if (registry?.Packs is null or { Count: 0 })
                {
                    if (!json)
                    {
                        AnsiConsole.MarkupLine("[yellow]No packs found in registry.[/]");
                    }
                    return;
                }

                // Filter packs
                var packs = registry.Packs
                    .Where(p =>
                    {
                        if (!string.IsNullOrEmpty(type) && p.Type != type)
                            return false;

                        if (!string.IsNullOrEmpty(search))
                        {
                            var searchLower = search.ToLowerInvariant();
                            if (!p.Name.Contains(search, StringComparison.OrdinalIgnoreCase) &&
                                !p.Id.Contains(search, StringComparison.OrdinalIgnoreCase))
                                return false;
                        }

                        return true;
                    })
                    .OrderBy(p => p.Type)
                    .ThenBy(p => p.Name)
                    .ToList();

                if (json)
                {
                    CommandOutput.WriteJson(packs);
                    return;
                }

                DisplayPackTable(packs);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[red]Error fetching registry: {Markup.Escape(ex.Message)}[/]");
            }
        });

        return command;
    }

    /// <summary>
    /// Creates the <c>registry search</c> subcommand.
    /// </summary>
    private static Command CreateSearchCommand()
    {
        Command command = new("search", "Search the pack registry");

        Argument<string> queryArg = new("query") { Description = "Search term" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        command.Add(queryArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string query = parseResult.GetRequiredValue(queryArg);
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var registry = await FetchRegistryAsync(ct).ConfigureAwait(false);

                if (registry?.Packs is null or { Count: 0 })
                {
                    if (!json)
                    {
                        AnsiConsole.MarkupLine("[yellow]No packs found in registry.[/]");
                    }
                    return;
                }

                var queryLower = query.ToLowerInvariant();
                var results = registry.Packs
                    .Where(p =>
                        p.Name.Contains(query, StringComparison.OrdinalIgnoreCase) ||
                        p.Id.Contains(query, StringComparison.OrdinalIgnoreCase))
                    .OrderBy(p => p.Type)
                    .ThenBy(p => p.Name)
                    .ToList();

                if (results.Count == 0)
                {
                    if (!json)
                    {
                        AnsiConsole.MarkupLine($"[yellow]No packs found matching '[bold]{Markup.Escape(query)}[/]'[/]");
                    }
                    return;
                }

                if (json)
                {
                    CommandOutput.WriteJson(results);
                    return;
                }

                DisplayPackTable(results);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[red]Error searching registry: {Markup.Escape(ex.Message)}[/]");
            }
        });

        return command;
    }

    /// <summary>
    /// Creates the <c>registry info</c> subcommand.
    /// </summary>
    private static Command CreateInfoCommand()
    {
        Command command = new("info", "Show detailed info about a pack");

        Argument<string> packIdArg = new("pack-id") { Description = "Pack ID" };
        Option<string> formatOpt = CommandOutput.CreateFormatOption();

        command.Add(packIdArg);
        command.Add(formatOpt);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            string packId = parseResult.GetRequiredValue(packIdArg);
            bool json = CommandOutput.IsJson(parseResult, formatOpt);

            try
            {
                var registry = await FetchRegistryAsync(ct).ConfigureAwait(false);
                var pack = registry?.Packs?.FirstOrDefault(p => p.Id == packId);

                if (pack is null)
                {
                    if (!json)
                    {
                        AnsiConsole.MarkupLine($"[red]Pack not found: {Markup.Escape(packId)}[/]");
                    }
                    return;
                }

                if (json)
                {
                    CommandOutput.WriteJson(pack);
                    return;
                }

                var table = new Table()
                    .Border(TableBorder.Rounded)
                    .Title($"[bold]{Markup.Escape(pack.Name)}[/]")
                    .AddColumn("Property")
                    .AddColumn("Value");

                table.AddRow("ID", Markup.Escape(pack.Id));
                table.AddRow("Version", Markup.Escape(pack.Version));
                table.AddRow("Type", Markup.Escape(pack.Type));
                table.AddRow("Author", Markup.Escape(pack.Author));
                table.AddRow("Framework", Markup.Escape(pack.FrameworkVersion));
                table.AddRow("Units", pack.UnitCount.ToString());
                table.AddRow("Buildings", pack.BuildingCount.ToString());
                table.AddRow("Factions", pack.FactionCount.ToString());
                table.AddRow("Screenshots", pack.ScreenshotCount.ToString());
                table.AddRow("URL", Markup.Escape(pack.Url));

                AnsiConsole.Write(table);
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[red]Error fetching pack info: {Markup.Escape(ex.Message)}[/]");
            }
        });

        return command;
    }

    /// <summary>
    /// Fetches the pack registry from GitHub Pages.
    /// </summary>
    private static async Task<PackRegistry?> FetchRegistryAsync(CancellationToken ct)
    {
        // Try local first (during development), then GitHub Pages
        string[] registryUrls = new[]
        {
            "https://kooshapari.github.io/Dino/packs/registry.json",
            "http://127.0.0.1:5173/packs/registry.json", // VitePress dev server
        };

        using var http = new HttpClient { Timeout = TimeSpan.FromSeconds(5) };

        foreach (var url in registryUrls)
        {
            try
            {
                var response = await http.GetAsync(url, ct).ConfigureAwait(false);
                if (!response.IsSuccessStatusCode)
                    continue;

                var json = await response.Content.ReadAsStringAsync(ct).ConfigureAwait(false);
                return JsonSerializer.Deserialize<PackRegistry>(json, JsonOptions.Default);
            }
            catch
            {
                // Try next URL
                continue;
            }
        }

        throw new HttpRequestException("Unable to fetch pack registry from any source");
    }

    /// <summary>
    /// Displays packs in a formatted table.
    /// </summary>
    private static void DisplayPackTable(IEnumerable<PackInfo> packs)
    {
        var table = new Table()
            .Border(TableBorder.Rounded)
            .Title("[bold]DINOForge Pack Registry[/]")
            .AddColumn("Name")
            .AddColumn("Type")
            .AddColumn("Version")
            .AddColumn("Author")
            .AddColumn("Content");

        foreach (var pack in packs)
        {
            var contentSummary = new List<string>();
            if (pack.UnitCount > 0) contentSummary.Add($"{pack.UnitCount} units");
            if (pack.BuildingCount > 0) contentSummary.Add($"{pack.BuildingCount} bldgs");
            if (pack.FactionCount > 0) contentSummary.Add($"{pack.FactionCount} factions");

            var typeColor = pack.Type switch
            {
                "total_conversion" => "cyan",
                "balance" => "green",
                "scenario" => "yellow",
                "utility" => "magenta",
                _ => "white"
            };

            table.AddRow(
                Markup.Escape(pack.Name),
                $"[{typeColor}]{Markup.Escape(pack.Type)}[/]",
                pack.Version,
                Markup.Escape(pack.Author),
                string.Join(", ", contentSummary));
        }

        AnsiConsole.Write(table);
    }

    /// <summary>
    /// Represents the pack registry JSON structure.
    /// </summary>
    private class PackRegistry
    {
        public string? Version { get; set; }
        public string? Generated { get; set; }
        public List<PackInfo> Packs { get; set; } = new();
    }

    /// <summary>
    /// Represents a pack entry in the registry.
    /// </summary>
    private class PackInfo
    {
        public string Id { get; set; } = string.Empty;
        public string Name { get; set; } = string.Empty;
        public string Version { get; set; } = string.Empty;
        public string Type { get; set; } = string.Empty;
        public string Author { get; set; } = string.Empty;
        public string FrameworkVersion { get; set; } = string.Empty;
        public int UnitCount { get; set; }
        public int BuildingCount { get; set; }
        public int FactionCount { get; set; }
        public int ScreenshotCount { get; set; }
        public string Url { get; set; } = string.Empty;
    }
}
