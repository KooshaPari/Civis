#nullable enable
using System;
using System.CommandLine;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using Microsoft.Extensions.Logging;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Assetctl;

/// <summary>
/// CLI command group for asset library browsing operations.
/// </summary>
public static class AssetLibraryCommand
{
    private static readonly string DefaultCatalogPath = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
        "DINOForge", "asset_catalog.db");

    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true
    };

    /// <summary>
    /// Creates the asset-library command group.
    /// </summary>
    /// <param name="logger">Optional logger for diagnostics.</param>
    /// <returns>The configured command.</returns>
    public static Command Create(ILogger? logger = null)
    {
        Command command = new("library", "Asset library browser and catalog management.");

        command.Add(CreateListCommand(logger));
        command.Add(CreateSearchCommand(logger));
        command.Add(CreateShowCommand(logger));
        command.Add(CreateStatsCommand(logger));
        command.Add(CreateImportCommand(logger));
        command.Add(CreateExportCommand(logger));
        command.Add(CreateSyncCommand(logger));

        return command;
    }

    private static Command CreateListCommand(ILogger? logger)
    {
        Option<string> factionOption = new("--faction")
        {
            Description = "Filter by faction (e.g., republic, cis, neutral)"
        };

        Option<string> typeOption = new("--type")
        {
            Description = "Filter by type (e.g., unit, vehicle, building, prop)"
        };

        Option<string> statusOption = new("--status")
        {
            Description = "Filter by status (e.g., normalized, validated, ready_for_prototype)"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "table",
            Description = "Output format: table | json"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Option<int> limitOption = new("--limit")
        {
            DefaultValueFactory = _ => 50,
            Description = "Maximum number of results to return"
        };

        Command listCommand = new("list", "List assets from the catalog.");
        listCommand.Add(factionOption);
        listCommand.Add(typeOption);
        listCommand.Add(statusOption);
        listCommand.Add(formatOption);
        listCommand.Add(catalogOption);
        listCommand.Add(limitOption);

        listCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string? faction = parseResult.GetValue(factionOption);
            string? type = parseResult.GetValue(typeOption);
            string? status = parseResult.GetValue(statusOption);
            string outputFormat = parseResult.GetValue(formatOption) ?? "table";
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;
            int limit = parseResult.GetValue(limitOption);

            if (!EnsureCatalogExists(catalogPath, logger))
            {
                return;
            }

            using var store = new AssetCatalogStore(catalogPath, null);
            var assets = store.SearchAssets(null, faction, type, status).Take(limit).ToList();

            if (!IsJsonOutput(outputFormat))
            {
                if (assets.Count == 0)
                {
                    AnsiConsole.MarkupLine("[yellow]No assets found matching the specified filters.[/]");
                    return;
                }

                Table table = new Table()
                    .Border(TableBorder.Rounded)
                    .Title("[bold]Asset Library[/]")
                    .AddColumn("Asset ID")
                    .AddColumn("Name")
                    .AddColumn("Faction")
                    .AddColumn("Type")
                    .AddColumn("Status");

                foreach (var asset in assets)
                {
                    table.AddRow(
                        Markup.Escape(asset.AssetId),
                        Markup.Escape(asset.Name ?? "-"),
                        Markup.Escape(asset.Faction ?? "-"),
                        Markup.Escape(asset.Type ?? "-"),
                        Markup.Escape(asset.Status ?? "-"));
                }

                AnsiConsole.Write(table);
                AnsiConsole.MarkupLine($"[dim]Showing {assets.Count} of {store.GetStats().TotalCount} total assets[/]");
                return;
            }

            WriteJson(new
            {
                success = true,
                command = "list",
                count = assets.Count,
                filters = new { faction, type, status },
                assets = assets.Select(a => new
                {
                    asset_id = a.AssetId,
                    name = a.Name,
                    faction = a.Faction,
                    type = a.Type,
                    status = a.Status,
                    provenance = a.Provenance,
                    pack_id = a.PackId,
                    created_at = a.CreatedAt.ToString("O")
                })
            });
        });

        return listCommand;
    }

    private static Command CreateSearchCommand(ILogger? logger)
    {
        Argument<string> queryArg = new("query")
        {
            Description = "Search query text"
        };

        Option<string> factionOption = new("--faction")
        {
            Description = "Filter by faction"
        };

        Option<string> typeOption = new("--type")
        {
            Description = "Filter by type"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "table",
            Description = "Output format: table | json"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Option<int> limitOption = new("--limit")
        {
            DefaultValueFactory = _ => 20,
            Description = "Maximum number of results to return"
        };

        Command searchCommand = new("search", "Search assets in the catalog.");
        searchCommand.Add(queryArg);
        searchCommand.Add(factionOption);
        searchCommand.Add(typeOption);
        searchCommand.Add(formatOption);
        searchCommand.Add(catalogOption);
        searchCommand.Add(limitOption);

        searchCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string query = parseResult.GetRequiredValue(queryArg);
            string? faction = parseResult.GetValue(factionOption);
            string? type = parseResult.GetValue(typeOption);
            string outputFormat = parseResult.GetValue(formatOption) ?? "table";
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;
            int limit = parseResult.GetValue(limitOption);

            if (!EnsureCatalogExists(catalogPath, logger))
            {
                return;
            }

            using var store = new AssetCatalogStore(catalogPath, null);
            var assets = store.SearchAssets(query, faction, type, null).Take(limit).ToList();

            if (!IsJsonOutput(outputFormat))
            {
                if (assets.Count == 0)
                {
                    AnsiConsole.MarkupLine($"[yellow]No assets found matching query '{Markup.Escape(query)}'.[/]");
                    return;
                }

                Table table = new Table()
                    .Border(TableBorder.Rounded)
                    .Title($"[bold]Search Results for '{Markup.Escape(query)}'[/]")
                    .AddColumn("Asset ID")
                    .AddColumn("Name")
                    .AddColumn("Faction")
                    .AddColumn("Type")
                    .AddColumn("Status");

                foreach (var asset in assets)
                {
                    table.AddRow(
                        Markup.Escape(asset.AssetId),
                        Markup.Escape(asset.Name ?? "-"),
                        Markup.Escape(asset.Faction ?? "-"),
                        Markup.Escape(asset.Type ?? "-"),
                        Markup.Escape(asset.Status ?? "-"));
                }

                AnsiConsole.Write(table);
                AnsiConsole.MarkupLine($"[dim]Found {assets.Count} result(s)[/]");
                return;
            }

            WriteJson(new
            {
                success = true,
                command = "search",
                query,
                count = assets.Count,
                assets = assets.Select(a => new
                {
                    asset_id = a.AssetId,
                    name = a.Name,
                    faction = a.Faction,
                    type = a.Type,
                    status = a.Status,
                    provenance = a.Provenance,
                    pack_id = a.PackId
                })
            });
        });

        return searchCommand;
    }

    private static Command CreateShowCommand(ILogger? logger)
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset identifier"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "table",
            Description = "Output format: table | json"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Command showCommand = new("show", "Show detailed information about an asset.");
        showCommand.Add(assetIdArg);
        showCommand.Add(formatOption);
        showCommand.Add(catalogOption);

        showCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string outputFormat = parseResult.GetValue(formatOption) ?? "table";
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;

            if (!EnsureCatalogExists(catalogPath, logger))
            {
                return;
            }

            using var store = new AssetCatalogStore(catalogPath, null);
            var asset = store.GetById(assetId);

            if (asset is null)
            {
                WriteError("show", $"Asset '{assetId}' not found in catalog", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                var panel = new Panel(new Table()
                    .Border(TableBorder.None)
                    .AddColumn("[dim]Property[/]")
                    .AddColumn("[bold]Value[/]")
                    .AddRow("Asset ID", Markup.Escape(asset.AssetId))
                    .AddRow("Name", Markup.Escape(asset.Name ?? "-"))
                    .AddRow("Faction", Markup.Escape(asset.Faction ?? "-"))
                    .AddRow("Type", Markup.Escape(asset.Type ?? "-"))
                    .AddRow("Status", Markup.Escape(asset.Status ?? "-"))
                    .AddRow("Provenance", Markup.Escape(asset.Provenance ?? "-"))
                    .AddRow("Pack ID", Markup.Escape(asset.PackId ?? "-"))
                    .AddRow("Source URL", Markup.Escape(asset.SourceUrl ?? "-"))
                    .AddRow("Created At", asset.CreatedAt.ToString("yyyy-MM-dd HH:mm:ss UTC", CultureInfo.InvariantCulture)))
                    .Header($"[bold]Asset: {Markup.Escape(asset.AssetId)}[/]")
                    .BorderColor(Color.Blue);

                AnsiConsole.Write(panel);

                if (!string.IsNullOrEmpty(asset.MetadataJson))
                {
                    AnsiConsole.MarkupLine("[dim]Metadata:[/]");
                    try
                    {
                        using var doc = JsonDocument.Parse(asset.MetadataJson);
                        var options = new JsonSerializerOptions { WriteIndented = true };
                        AnsiConsole.Write(new Panel(JsonSerializer.Serialize(doc.RootElement, options))
                            .BorderColor(Color.Grey));
                    }
                    catch
                    {
                        AnsiConsole.MarkupLine($"[dim]{Markup.Escape(asset.MetadataJson ?? string.Empty)}[/]");
                    }
                }

                return;
            }

            WriteJson(new
            {
                success = true,
                command = "show",
                asset = new
                {
                    asset_id = asset.AssetId,
                    name = asset.Name,
                    faction = asset.Faction,
                    type = asset.Type,
                    status = asset.Status,
                    provenance = asset.Provenance,
                    pack_id = asset.PackId,
                    source_url = asset.SourceUrl,
                    created_at = asset.CreatedAt.ToString("O"),
                    metadata = string.IsNullOrEmpty(asset.MetadataJson) ? null : JsonSerializer.Deserialize<object>(asset.MetadataJson)
                }
            });
        });

        return showCommand;
    }

    private static Command CreateStatsCommand(ILogger? logger)
    {
        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "table",
            Description = "Output format: table | json"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Command statsCommand = new("stats", "Show catalog statistics.");
        statsCommand.Add(formatOption);
        statsCommand.Add(catalogOption);

        statsCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string outputFormat = parseResult.GetValue(formatOption) ?? "table";
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;

            if (!EnsureCatalogExists(catalogPath, logger))
            {
                return;
            }

            using var store = new AssetCatalogStore(catalogPath, null);
            var stats = store.GetStats();

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine($"[bold]Asset Catalog Statistics[/]");
                AnsiConsole.MarkupLine($"Total assets: [green]{stats.TotalCount}[/]");

                if (stats.CountByFaction.Count > 0)
                {
                    AnsiConsole.MarkupLine("\n[bold]By Faction:[/]");
                    foreach (var kvp in stats.CountByFaction.OrderByDescending(x => x.Value))
                    {
                        AnsiConsole.MarkupLine($"  {kvp.Key}: [cyan]{kvp.Value}[/]");
                    }
                }

                if (stats.CountByType.Count > 0)
                {
                    AnsiConsole.MarkupLine("\n[bold]By Type:[/]");
                    foreach (var kvp in stats.CountByType.OrderByDescending(x => x.Value))
                    {
                        AnsiConsole.MarkupLine($"  {kvp.Key}: [cyan]{kvp.Value}[/]");
                    }
                }

                if (stats.CountByStatus.Count > 0)
                {
                    AnsiConsole.MarkupLine("\n[bold]By Status:[/]");
                    foreach (var kvp in stats.CountByStatus.OrderByDescending(x => x.Value))
                    {
                        AnsiConsole.MarkupLine($"  {kvp.Key}: [cyan]{kvp.Value}[/]");
                    }
                }

                if (stats.CountByPack.Count > 0)
                {
                    AnsiConsole.MarkupLine("\n[bold]By Pack:[/]");
                    foreach (var kvp in stats.CountByPack.OrderByDescending(x => x.Value))
                    {
                        AnsiConsole.MarkupLine($"  {kvp.Key}: [cyan]{kvp.Value}[/]");
                    }
                }

                return;
            }

            WriteJson(new
            {
                success = true,
                command = "stats",
                total_count = stats.TotalCount,
                count_by_faction = stats.CountByFaction,
                count_by_type = stats.CountByType,
                count_by_status = stats.CountByStatus,
                count_by_pack = stats.CountByPack
            });
        });

        return statsCommand;
    }

    private static Command CreateImportCommand(ILogger? logger)
    {
        Argument<string> sourceArg = new("source")
        {
            Description = "Import source: local (from pack registries) or json:<path>"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Option<string> repoRootOption = new("--repo-root")
        {
            DefaultValueFactory = _ => GetRepoRoot(),
            Description = "Repository root path for local imports"
        };

        Command importCommand = new("import", "Import assets into the catalog.");
        importCommand.Add(sourceArg);
        importCommand.Add(catalogOption);
        importCommand.Add(repoRootOption);

        importCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string source = parseResult.GetRequiredValue(sourceArg);
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;
            string repoRoot = parseResult.GetValue(repoRootOption) ?? GetRepoRoot();

            EnsureCatalogDirectory(catalogPath, logger);
            using var store = new AssetCatalogStore(catalogPath, null);

            int count = 0;

            if (source.StartsWith("local:", StringComparison.OrdinalIgnoreCase))
            {
                count = await ImportFromLocalAsync(store, repoRoot, logger, ct);
            }
            else if (source.StartsWith("json:", StringComparison.OrdinalIgnoreCase))
            {
                string jsonPath = source.Substring(5);
                count = store.ImportFromJson(jsonPath);
            }
            else if (source.Equals("local", StringComparison.OrdinalIgnoreCase))
            {
                count = await ImportFromLocalAsync(store, repoRoot, logger, ct);
            }
            else if (File.Exists(source))
            {
                count = store.ImportFromJson(source);
            }
            else
            {
                AnsiConsole.MarkupLine($"[red]Unknown import source: {Markup.Escape(source)}[/]");
                return;
            }

            AnsiConsole.MarkupLine($"[green]✓[/] Imported {count} assets into catalog.");
        });

        return importCommand;
    }

    private static Command CreateExportCommand(ILogger? logger)
    {
        Argument<string> exportPathArg = new("exportPath")
        {
            Description = "Path to export JSON file"
        };

        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Command exportCommand = new("export", "Export catalog to JSON.");
        exportCommand.Add(exportPathArg);
        exportCommand.Add(catalogOption);

        exportCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string exportPath = parseResult.GetRequiredValue(exportPathArg);
            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;

            if (!EnsureCatalogExists(catalogPath, logger))
            {
                return;
            }

            using var store = new AssetCatalogStore(catalogPath, null);
            bool success = store.ExportToJson(exportPath);

            if (success)
            {
                AnsiConsole.MarkupLine($"[green]✓[/] Exported catalog to [bold]{Markup.Escape(exportPath)}[/]");
            }
            else
            {
                AnsiConsole.MarkupLine("[red]✗[/] Export failed.");
            }
        });

        return exportCommand;
    }

    private static Command CreateSyncCommand(ILogger? logger)
    {
        Option<string> catalogOption = new("--catalog")
        {
            DefaultValueFactory = _ => DefaultCatalogPath,
            Description = "Path to the catalog database"
        };

        Option<string> repoRootOption = new("--repo-root")
        {
            DefaultValueFactory = _ => GetRepoRoot(),
            Description = "Repository root path"
        };

        Option<bool> forceOption = new("--force")
        {
            Description = "Force re-import even if asset already exists"
        };

        Command syncCommand = new("sync", "Sync assets from pack registries into the catalog.");
        syncCommand.Add(catalogOption);
        syncCommand.Add(repoRootOption);
        syncCommand.Add(forceOption);

        syncCommand.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();

            string catalogPath = parseResult.GetValue(catalogOption) ?? DefaultCatalogPath;
            string repoRoot = parseResult.GetValue(repoRootOption) ?? GetRepoRoot();
            bool force = parseResult.GetValue(forceOption);

            EnsureCatalogDirectory(catalogPath, logger);
            using var store = new AssetCatalogStore(catalogPath, null);

            int count = await ImportFromLocalAsync(store, repoRoot, logger, ct, force);

            AnsiConsole.MarkupLine($"[green]✓[/] Synced {count} assets into catalog.");
        });

        return syncCommand;
    }

    private static async Task<int> ImportFromLocalAsync(
        AssetCatalogStore store,
        string repoRoot,
        ILogger? logger,
        CancellationToken ct,
        bool force = false)
    {
        int count = 0;

        var localAdapter = new LocalSourceAdapter(repoRoot, null);
        IEnumerable<AssetCandidate> candidates = await localAdapter.SearchAsync("", ct);

        foreach (var candidate in candidates)
        {
            ct.ThrowIfCancellationRequested();

            var entry = new AssetCatalogEntry
            {
                AssetId = candidate.ExternalId,
                Name = candidate.Title,
                Faction = DetermineFaction(candidate.Tags),
                Type = candidate.Category,
                SourceUrl = candidate.SourceUrl,
                Status = candidate.IpStatus,
                Provenance = "local_registry",
                PackId = GetPackIdFromTags(candidate.Tags),
                CreatedAt = DateTime.UtcNow
            };

            if (force)
            {
                store.AddAsset(entry);
                count++;
            }
            else
            {
                if (store.AddAsset(entry))
                {
                    count++;
                }
            }
        }

        return count;
    }

    private static string? DetermineFaction(string[]? tags)
    {
        if (tags is null)
        {
            return null;
        }

        foreach (var tag in tags)
        {
            string lowered = tag.ToLowerInvariant();
            if (lowered is "republic" or "cis" or "empire" or "rebel" or "neutral")
            {
                return lowered;
            }
        }

        return null;
    }

    private static string? GetPackIdFromTags(string[]? tags)
    {
        if (tags is null || tags.Length == 0)
        {
            return null;
        }

        // Assume first tag that doesn't look like a faction or type
        foreach (var tag in tags)
        {
            string lowered = tag.ToLowerInvariant();
            if (lowered is not "republic" and not "cis" and not "empire" and not "rebel" and not "neutral" and not "infantry" and not "vehicle")
            {
                return tag;
            }
        }

        return null;
    }

    private static bool EnsureCatalogExists(string catalogPath, ILogger? logger)
    {
        if (File.Exists(catalogPath))
        {
            return true;
        }

        EnsureCatalogDirectory(catalogPath, logger);

        // Create empty catalog
        try
        {
            using var store = new AssetCatalogStore(catalogPath, null);
            logger?.LogInformation("Created new catalog at {Path}", catalogPath);
            return true;
        }
        catch (Exception ex)
        {
            logger?.LogError(ex, "Failed to create catalog at {Path}", catalogPath);
            return false;
        }
    }

    private static void EnsureCatalogDirectory(string catalogPath, ILogger? logger)
    {
        string? directory = Path.GetDirectoryName(catalogPath);
        if (!string.IsNullOrEmpty(directory) && !Directory.Exists(directory))
        {
            try
            {
                Directory.CreateDirectory(directory);
                logger?.LogInformation("Created catalog directory at {Path}", directory);
            }
            catch (Exception ex)
            {
                logger?.LogError(ex, "Failed to create catalog directory at {Path}", directory);
            }
        }
    }

    private static string GetRepoRoot()
    {
        // Try to determine repository root from current directory
        string? dir = Directory.GetCurrentDirectory();
        while (dir is not null)
        {
            if (File.Exists(Path.Combine(dir, "pack.yaml")) ||
                File.Exists(Path.Combine(dir, "CLAUDE.md")) ||
                Directory.Exists(Path.Combine(dir, "packs")))
            {
                return dir;
            }

            dir = Path.GetDirectoryName(dir);
        }

        return Directory.GetCurrentDirectory();
    }

    private static bool IsJsonOutput(string outputFormat)
    {
        return string.Equals(outputFormat, "json", StringComparison.OrdinalIgnoreCase);
    }

    private static void WriteError(string command, string message, string outputFormat)
    {
        if (IsJsonOutput(outputFormat))
        {
            WriteJson(new { success = false, command, message });
            return;
        }

        AnsiConsole.MarkupLine($"[red]✗[/] {Markup.Escape(command)}: {Markup.Escape(message)}");
    }

    private static void WriteJson(object payload)
    {
        Console.WriteLine(JsonSerializer.Serialize(payload, JsonOptions));
    }
}
