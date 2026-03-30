using System;
using System.Globalization;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.CommandLine;
using DINOForge.Tools.Cli.Assetctl.Sketchfab;
using Microsoft.Extensions.DependencyInjection;
using Spectre.Console;

// CS1998: These async lambdas are required by System.CommandLine's SetAction API.
// The lambdas are synchronous but the API signature requires async delegate.
#pragma warning disable CS1998

namespace DINOForge.Tools.Cli.Assetctl;

internal static class AssetctlCommand
{
    private static readonly string[] AllowedPipelineBundles = ["unit", "vehicle", "prop"];
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = true
    };

    public static Command Create(IServiceProvider serviceProvider)
    {
        Command command = new("assetctl", "Agent-friendly asset intake pipeline commands.");
        // DIAGNOSTIC: Isolating which command causes hang
        command.Add(CreateSearchCommand());
        command.Add(CreateIntakeCommand());
        command.Add(CreateNormalizeCommand());
        command.Add(CreateValidateCommand());
        command.Add(CreateStylizeCommand());
        command.Add(CreateRegisterCommand());
        command.Add(CreateExportUnityCommand());
        command.Add(CreateSearchSketchfabCommand(serviceProvider));
        command.Add(CreateDownloadSketchfabCommand(serviceProvider));
        command.Add(CreateDownloadBatchSketchfabCommand(serviceProvider));
        command.Add(CreateValidateSketchfabTokenCommand(serviceProvider));
        command.Add(CreateSketchfabQuotaCommand(serviceProvider));
        command.Add(AssetLibraryCommand.Create());
        return command;
    }

    private static Command CreateSearchCommand()
    {
        Argument<string> queryArg = new("query")
        {
            Description = "Search query text"
        };

        Option<string> sourceOption = new("--source")
        {
            DefaultValueFactory = _ => "all",
            Description = "Source tier to search (for example: sketchfab, blendswap, moddb)"
        };

        Option<int> limitOption = new("--limit")
        {
            DefaultValueFactory = _ => 10,
            Description = "Maximum number of results to return"
        };

        Option<string> licenseOption = new("--license")
        {
            DefaultValueFactory = _ => "all",
            Description = "License filter (optional)"
        };

        Option<int?> minPolyOption = new("--min-poly")
        {
            Description = "Minimum polygon estimate"
        };

        Option<int?> maxPolyOption = new("--max-poly")
        {
            Description = "Maximum polygon estimate"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Command command = new("search", "Search candidates from configured intake sources.");
        command.Add(queryArg);
        command.Add(sourceOption);
        command.Add(limitOption);
        command.Add(licenseOption);
        command.Add(minPolyOption);
        command.Add(maxPolyOption);
        command.Add(formatOption);
        command.Add(pipelineRootOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string query = parseResult.GetRequiredValue(queryArg);
            string source = parseResult.GetValue(sourceOption) ?? "all";
            int limit = parseResult.GetValue(limitOption);
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            string licenseFilter = parseResult.GetValue(licenseOption) ?? "all";
            int? minPoly = parseResult.GetValue(minPolyOption);
            int? maxPoly = parseResult.GetValue(maxPolyOption);
            _ = parseResult.GetValue(pipelineRootOption);

            AssetctlPipeline pipeline = new();
            AssetctlSearchFilters filters = new()
            {
                License = licenseFilter,
                MinPoly = minPoly,
                MaxPoly = maxPoly
            };

            System.Collections.Generic.IReadOnlyList<RankedCandidate> results = pipeline.Search(query, source, limit, filters);
            if (!IsJsonOutput(outputFormat))
            {
                Table table = new Table()
                    .Border(TableBorder.Rounded)
                    .Title("[bold]Asset Search[/]")
                    .AddColumn("Rank")
                    .AddColumn("Source")
                    .AddColumn("Title")
                    .AddColumn("License")
                    .AddColumn("Poly")
                    .AddColumn("Score");

                for (int i = 0; i < results.Count; i++)
                {
                    RankedCandidate result = results[i];
                    table.AddRow(
                        (i + 1).ToString(CultureInfo.InvariantCulture),
                        result.Candidate.Source,
                        Markup.Escape(result.Candidate.Title),
                        Markup.Escape(result.Candidate.LicenseLabel ?? "unknown"),
                        result.Candidate.PolycountEstimate?.ToString(CultureInfo.InvariantCulture) ?? "n/a",
                        result.Score.ToString("0.00", CultureInfo.InvariantCulture));
                }

                AnsiConsole.Write(table);
                AnsiConsole.MarkupLine($"[dim]Found {results.Count} result(s) for query '{Markup.Escape(query)}'[/]");
                return;
            }

            WriteJson(new
            {
                success = true,
                command = "search",
                query,
                source,
                filters = new
                {
                    license = licenseFilter,
                    min_poly = minPoly,
                    max_poly = maxPoly,
                    limit
                },
                results_count = results.Count,
                results = results.Select(item => new
                {
                    source = item.Candidate.Source,
                    external_id = item.Candidate.ExternalId,
                    title = item.Candidate.Title,
                    author_name = item.Candidate.AuthorName,
                    license_label = item.Candidate.LicenseLabel,
                    original_format = item.Candidate.OriginalFormat,
                    score = item.Score,
                    score_breakdown = item.ScoreBreakdown,
                    category = item.Candidate.Category,
                    subtype = item.Candidate.Subtype,
                    ip_status = item.Candidate.IpStatus,
                    polycount_estimate = item.Candidate.PolycountEstimate
                })
            });

            return;
        });

        return command;
    }

    private static Command CreateIntakeCommand()
    {
        Argument<string> candidateRefArg = new("candidateRef")
        {
            Description = "Candidate reference in format <source>:<externalId>"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("intake", "Create raw intake artifacts for a search candidate.");
        command.Add(candidateRefArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string candidateRef = parseResult.GetRequiredValue(candidateRefArg);
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";

            if (!TryParseCandidateRef(candidateRef, out string source, out string externalId, out string parseError))
            {
                WriteError("intake", parseError, outputFormat);
                return;
            }

            AssetctlPipeline pipeline = new();
            AssetctlIntakeResult result = pipeline.Intake(source, externalId, pipelineRoot);
            if (!result.Success)
            {
                WriteError("intake", result.Message ?? "intake failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Intake created.");
                AnsiConsole.MarkupLine($"  Asset ID: [bold]{Markup.Escape(result.AssetId ?? string.Empty)}[/]");
                AnsiConsole.MarkupLine($"  Manifest: [dim]{Markup.Escape(result.ManifestPath ?? string.Empty)}[/]");
                if (!string.IsNullOrEmpty(result.RawDir))
                {
                    AnsiConsole.MarkupLine($"  Directory: [dim]{Markup.Escape(result.RawDir)}[/]");
                }

                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateNormalizeCommand()
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset pipeline identifier"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Option<string?> blenderPathOption = new("--blender-path")
        {
            Description = "Path to Blender executable (overrides env var and auto-detection)"
        };

        Option<int> targetPolycountOption = new("--target-polycount")
        {
            DefaultValueFactory = _ => 3000,
            Description = "Target polycount for optimization (default: 3000)"
        };

        Command command = new("normalize", "Normalize a candidate into working output.");
        command.Add(assetIdArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);
        command.Add(blenderPathOption);
        command.Add(targetPolycountOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            string? blenderPath = parseResult.GetValue(blenderPathOption);
            int targetPolycount = parseResult.GetValue(targetPolycountOption);

            AssetctlPipeline pipeline = new();
            AssetctlNormalizeResult result = pipeline.Normalize(assetId, pipelineRoot, blenderPath, targetPolycount);
            if (!result.Success)
            {
                WriteError("normalize", result.Message ?? "normalize failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Asset normalized.");
                AnsiConsole.MarkupLine($"  Asset ID: [bold]{Markup.Escape(result.AssetId ?? string.Empty)}[/]");
                AnsiConsole.MarkupLine($"  Working Dir: [dim]{Markup.Escape(result.WorkingDir ?? string.Empty)}[/]");
                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateValidateCommand()
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset pipeline identifier"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("validate", "Validate manifest and schema compliance.");
        command.Add(assetIdArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            AssetctlPipeline pipeline = new();
            AssetctlValidationResult result = pipeline.Validate(assetId, pipelineRoot);
            if (!result.Success)
            {
                WriteError("validate", result.Message ?? "validate failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Asset validated.");
                AnsiConsole.MarkupLine($"  Asset ID: [bold]{Markup.Escape(result.AssetId ?? string.Empty)}[/]");
                AnsiConsole.MarkupLine($"  Technical status: [dim]{Markup.Escape(result.TechnicalStatus ?? "n/a")}[/]");
                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateStylizeCommand()
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset pipeline identifier"
        };

        Option<string> profileOption = new("--profile")
        {
            DefaultValueFactory = _ => "dinosw_lowpoly_v1",
            Description = "Stylization profile name"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Option<string?> factionOption = new("--faction")
        {
            Description = "Faction override: republic | cis | neutral"
        };

        Option<bool> dryRunOption = new("--dry-run")
        {
            Description = "Preview palette without running Blender"
        };

        Option<string?> blenderPathOption = new("--blender-path")
        {
            Description = "Path to Blender executable (overrides env var and auto-detection)"
        };

        Command command = new("stylize", "Add stylization metadata and mark the asset as prototype-ready.");
        command.Add(assetIdArg);
        command.Add(profileOption);
        command.Add(pipelineRootOption);
        command.Add(formatOption);
        command.Add(factionOption);
        command.Add(dryRunOption);
        command.Add(blenderPathOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string profile = parseResult.GetValue(profileOption) ?? "dinosw_lowpoly_v1";
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            string? faction = parseResult.GetValue(factionOption);
            bool dryRun = parseResult.GetValue(dryRunOption);
            string? blenderPath = parseResult.GetValue(blenderPathOption);

            AssetctlPipeline pipeline = new();
            AssetctlStylizeResult result = pipeline.Stylize(assetId, profile, pipelineRoot, faction, blenderPath, dryRun);
            if (!result.Success)
            {
                WriteError("stylize", result.Message ?? "stylize failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Asset stylized.");
                AnsiConsole.MarkupLine($"  Profile: [bold]{Markup.Escape(result.Profile ?? string.Empty)}[/]");
                if (!string.IsNullOrEmpty(result.DryRunPalette))
                {
                    AnsiConsole.MarkupLine($"  [dim]Palette (dry-run):[/]");
                    AnsiConsole.MarkupLine($"  {Markup.Escape(result.DryRunPalette)}");
                }
                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateRegisterCommand()
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset pipeline identifier"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("register", "Add an asset to the local registry index.");
        command.Add(assetIdArg);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            AssetctlPipeline pipeline = new();
            AssetctlRegisterResult result = pipeline.Register(assetId, pipelineRoot);
            if (!result.Success)
            {
                WriteError("register", result.Message ?? "register failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Asset added to registry.");
                AnsiConsole.MarkupLine($"  Total registered: [bold]{result.TotalRegistered}[/]");
                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateExportUnityCommand()
    {
        Argument<string> assetIdArg = new("assetId")
        {
            Description = "Asset pipeline identifier"
        };

        Option<string> bundleOption = new("--bundle")
        {
            DefaultValueFactory = _ => "unit",
            Description = "Unity bundle: unit, vehicle, prop"
        };

        Option<string> pipelineRootOption = new("--pipeline-root")
        {
            DefaultValueFactory = _ => "assets-pipeline",
            Description = "Pipeline root directory"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("export-unity", "Export the normalized asset to unity bundle output.");
        command.Add(assetIdArg);
        command.Add(bundleOption);
        command.Add(pipelineRootOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string assetId = parseResult.GetRequiredValue(assetIdArg);
            string pipelineRoot = parseResult.GetValue(pipelineRootOption) ?? "assets-pipeline";
            string bundle = parseResult.GetValue(bundleOption) ?? "unit";
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            if (!AllowedPipelineBundles.Contains(bundle, StringComparer.OrdinalIgnoreCase))
            {
                WriteError("export-unity", $"Unsupported bundle '{bundle}'", outputFormat);
                return;
            }

            AssetctlPipeline pipeline = new();
            AssetctlExportResult result = pipeline.ExportUnity(assetId, pipelineRoot, bundle);
            if (!result.Success)
            {
                WriteError("export-unity", result.Message ?? "export-unity failed", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[green]✓[/] Asset exported for Unity.");
                AnsiConsole.MarkupLine($"  Bundle: [bold]{Markup.Escape(result.Bundle ?? string.Empty)}[/]");
                AnsiConsole.MarkupLine($"  Directory: [dim]{Markup.Escape(result.ExportDir ?? string.Empty)}[/]");
                return;
            }

            WriteJson(result);
            return;
        });

        return command;
    }

    private static Command CreateSearchSketchfabCommand(IServiceProvider serviceProvider)
    {
        Argument<string> queryArg = new("query")
        {
            Description = "Search query text for Sketchfab"
        };

        Option<string> licenseOption = new("--license")
        {
            DefaultValueFactory = _ => ReadEnvironment("SKETCHFAB_LICENSE_FILTER", "cc0,cc-by,cc-by-sa"),
            Description = "CSV license filter: cc0,cc-by,cc-by-sa"
        };

        Option<int> maxPolyOption = new("--max-poly")
        {
            DefaultValueFactory = _ => ReadEnvironmentInt("SKETCHFAB_POLY_MAX", 5000),
            Description = "Maximum polycount estimate"
        };

        Option<int?> minPolyOption = new("--min-poly")
        {
            Description = "Minimum polycount estimate"
        };

        Option<string> sortByOption = new("--sort-by")
        {
            DefaultValueFactory = _ => ReadEnvironment("SKETCHFAB_SORT_BY", "relevance"),
            Description = "Sort order: relevance, likeCount, viewCount, publishedAt"
        };

        Option<bool> excludePaidOption = new("--exclude-paid")
        {
            DefaultValueFactory = _ => ReadEnvironmentBool("SKETCHFAB_EXCLUDE_PAID", true),
            Description = "Exclude paid or exclusive models"
        };

        Option<int> limitOption = new("--limit")
        {
            DefaultValueFactory = _ => 10,
            Description = "Maximum number of results"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("search-sketchfab", "Search candidate models on Sketchfab.");
        command.Add(queryArg);
        command.Add(licenseOption);
        command.Add(maxPolyOption);
        command.Add(minPolyOption);
        command.Add(sortByOption);
        command.Add(excludePaidOption);
        command.Add(limitOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string query = parseResult.GetRequiredValue(queryArg);
            string license = parseResult.GetValue(licenseOption) ?? "cc0,cc-by,cc-by-sa";
            int maxPoly = parseResult.GetValue(maxPolyOption);
            int? minPoly = parseResult.GetValue(minPolyOption);
            string sortBy = parseResult.GetValue(sortByOption) ?? "relevance";
            bool excludePaid = parseResult.GetValue(excludePaidOption);
            int limit = parseResult.GetValue(limitOption);
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            SketchfabClient client = serviceProvider.GetRequiredService<SketchfabClient>();
            AssetDownloader downloader = new(client, new AssetDownloaderOptions
            {
                DefaultLicenses = license,
                MaxPolyDefault = maxPoly
            });

            try
            {
                System.Collections.Generic.IReadOnlyList<Sketchfab.AssetCandidate> candidates = await downloader.SearchCandidatesAsync(
                    query,
                    new AssetSearchCriteria
                    {
                        AllowedLicenses = license,
                        MaxPolyCount = maxPoly,
                        MinPolyCount = minPoly,
                        SortBy = sortBy,
                        ExcludePaid = excludePaid,
                        MaxCandidates = limit
                    },
                    ct);

                if (!IsJsonOutput(outputFormat))
                {
                    Table table = new Table()
                        .Border(TableBorder.Rounded)
                        .Title("[bold]Sketchfab Search[/]")
                        .AddColumn("Name")
                        .AddColumn("License")
                        .AddColumn("Poly")
                        .AddColumn("Score");

                    foreach (Sketchfab.AssetCandidate candidate in candidates)
                    {
                        table.AddRow(
                            Markup.Escape(candidate.Name),
                            Markup.Escape(candidate.License ?? "unknown"),
                            candidate.PolyCount?.ToString(CultureInfo.InvariantCulture) ?? "n/a",
                            candidate.ConfidenceScore.ToString("0.00", CultureInfo.InvariantCulture));
                    }

                    AnsiConsole.Write(table);
                    AnsiConsole.MarkupLine($"[dim]Found {candidates.Count} candidate(s) for '{Markup.Escape(query)}'[/]");
                    return;
                }

                SketchfabRateLimitState? state = client.GetRateLimitState();
                WriteJson(new
                {
                    success = true,
                    command = "search-sketchfab",
                    query,
                    filters = new
                    {
                        license,
                        max_poly = maxPoly,
                        min_poly = minPoly,
                        sort_by = sortBy,
                        exclude_paid = excludePaid,
                        limit
                    },
                    results_count = candidates.Count,
                    results = candidates.Select(candidate => new
                    {
                        model_id = candidate.ModelId,
                        name = candidate.Name,
                        creator = candidate.Creator,
                        license = candidate.License,
                        poly_count = candidate.PolyCount,
                        published_at = candidate.PublishedAt?.ToString("O", CultureInfo.InvariantCulture),
                        model_url = candidate.ModelUrl,
                        confidence_score = candidate.ConfidenceScore,
                        ranking_details = candidate.RankingDetails
                    }),
                    rate_limit = state is null
                        ? null
                        : new
                        {
                            remaining = state.Remaining,
                            reset_at = state.ResetAtUtc.ToString("O", CultureInfo.InvariantCulture),
                            reset_in_seconds = state.ResetInSeconds
                        }
                });
            }
            catch (SketchfabApiException ex)
            {
                WriteError("search-sketchfab", $"Sketchfab API error: {ex.Message}", outputFormat);
            }
            catch (Exception ex)
            {
                WriteError("search-sketchfab", $"Search failed: {ex.Message}", outputFormat);
            }

            return;
        });

        return command;
    }

    private static Command CreateDownloadSketchfabCommand(IServiceProvider serviceProvider)
    {
        Argument<string> modelRefArg = new("modelRef")
        {
            Description = "Model reference in format sketchfab:<model_id>"
        };

        Option<string> outputDirOption = new("--output")
        {
            DefaultValueFactory = _ => ReadEnvironment("ASSET_DOWNLOAD_DIR", "packs/warfare-starwars/assets/raw"),
            Description = "Output directory"
        };

        Option<string> fileFormatOption = new("--file-format")
        {
            DefaultValueFactory = _ => ReadEnvironment("SKETCHFAB_DOWNLOAD_FORMAT", "glb"),
            Description = "Download format"
        };

        Option<bool> generateManifestOption = new("--generate-manifest")
        {
            DefaultValueFactory = _ => ReadEnvironmentBool("ASSET_AUTO_MANIFEST", true),
            Description = "Generate asset_manifest.json"
        };

        Option<string> franchiseOption = new("--franchise")
        {
            DefaultValueFactory = _ => "star_wars",
            Description = "Franchise tag used for generated asset IDs"
        };

        Option<string> ipStatusOption = new("--ip-status")
        {
            DefaultValueFactory = _ => "fan_star_wars_private_only",
            Description = "IP status label"
        };

        Option<string> outputFormatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("download-sketchfab", "Download a single Sketchfab model.");
        command.Add(modelRefArg);
        command.Add(outputDirOption);
        command.Add(fileFormatOption);
        command.Add(generateManifestOption);
        command.Add(franchiseOption);
        command.Add(ipStatusOption);
        command.Add(outputFormatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string modelRef = parseResult.GetRequiredValue(modelRefArg);
            string outputFormat = parseResult.GetValue(outputFormatOption) ?? "text";
            string outputDir = parseResult.GetValue(outputDirOption) ?? ReadEnvironment("ASSET_DOWNLOAD_DIR", "packs/warfare-starwars/assets/raw");
            string format = parseResult.GetValue(fileFormatOption) ?? "glb";
            bool generateManifest = parseResult.GetValue(generateManifestOption);
            string franchise = parseResult.GetValue(franchiseOption) ?? "star_wars";
            string ipStatus = parseResult.GetValue(ipStatusOption) ?? "fan_star_wars_private_only";

            if (!TryParseModelRef(modelRef, out string source, out string modelId, out string parseError))
            {
                WriteError("download-sketchfab", parseError, outputFormat);
                return;
            }

            if (!string.Equals(source, "sketchfab", StringComparison.OrdinalIgnoreCase))
            {
                WriteError("download-sketchfab", $"Unsupported source '{source}'. Expected 'sketchfab'.", outputFormat);
                return;
            }

            SketchfabClient client = serviceProvider.GetRequiredService<SketchfabClient>();
            AssetDownloader downloader = new(client, new AssetDownloaderOptions
            {
                DefaultLicenses = ReadEnvironment("SKETCHFAB_LICENSE_FILTER", "cc0,cc-by,cc-by-sa"),
                MaxPolyDefault = ReadEnvironmentInt("SKETCHFAB_POLY_MAX", 5000),
                FranchiseTag = franchise,
                IpStatusDefault = ipStatus,
                OutputDirectory = outputDir,
                GenerateHashes = true,
                AutoGenerateManifests = generateManifest
            });

            try
            {
                DownloadAssetResult result = await downloader.DownloadAssetAsync(
                    new Sketchfab.AssetCandidate
                    {
                        ModelId = modelId,
                        Name = modelId,
                        PolyCount = null,
                        PublishedAt = null
                    },
                    outputDir,
                    ct);

                if (result.Success)
                {
                    PatchManifestIp(result, ipStatus);

                    if (!IsJsonOutput(outputFormat))
                    {
                        AnsiConsole.MarkupLine("[green]✓[/] Sketchfab download complete.");
                        AnsiConsole.MarkupLine($"  Model ID: [bold]{Markup.Escape(modelId)}[/]");
                        AnsiConsole.MarkupLine($"  Asset ID: [dim]{Markup.Escape(result.AssetId)}[/]");
                        AnsiConsole.MarkupLine($"  File: [dim]{Markup.Escape(result.FilePath)}[/]");
                        AnsiConsole.MarkupLine($"  SHA256: [dim]{Markup.Escape(result.Sha256)}[/]");
                        AnsiConsole.MarkupLine($"  Format: [dim]{Markup.Escape(format)}[/]");
                        return;
                    }

                    WriteJson(new
                    {
                        success = true,
                        command = "download-sketchfab",
                        model_id = modelId,
                        format,
                        asset_id = result.AssetId,
                        file_path = result.FilePath,
                        manifest_path = result.ManifestPath,
                        metadata_path = result.MetadataPath,
                        file_size_bytes = result.FileSizeBytes,
                        sha256 = result.Sha256,
                        download_duration_ms = result.DurationMs,
                        download_speed_mbps = result.DurationMs > 0
                            ? ((result.FileSizeBytes * 8d / 1_000_000d) / (result.DurationMs / 1000d))
                            : 0
                    });

                    return;
                }

                WriteError("download-sketchfab", result.ErrorMessage ?? "Download failed", outputFormat);
            }
            catch (SketchfabApiException ex)
            {
                WriteError("download-sketchfab", $"Sketchfab API error: {ex.Message}", outputFormat);
            }
            catch (Exception ex)
            {
                WriteError("download-sketchfab", $"Download failed: {ex.Message}", outputFormat);
            }
        });

        return command;
    }

    private static Command CreateDownloadBatchSketchfabCommand(IServiceProvider serviceProvider)
    {
        Argument<string> queryArg = new("query")
        {
            Description = "Search query used to build batch candidate list"
        };

        Option<string> outputDirOption = new("--output")
        {
            DefaultValueFactory = _ => ReadEnvironment("ASSET_DOWNLOAD_DIR", "packs/warfare-starwars/assets/raw"),
            Description = "Output directory"
        };

        Option<int> limitOption = new("--limit")
        {
            DefaultValueFactory = _ => ReadEnvironmentInt("ASSET_BATCH_SIZE_MAX", 5),
            Description = "Maximum number of downloads"
        };

        Option<int> maxPolyOption = new("--max-poly")
        {
            DefaultValueFactory = _ => ReadEnvironmentInt("SKETCHFAB_POLY_MAX", 5000),
            Description = "Maximum polycount estimate"
        };

        Option<int?> minPolyOption = new("--min-poly")
        {
            Description = "Minimum polycount estimate"
        };

        Option<string> licenseOption = new("--license")
        {
            DefaultValueFactory = _ => ReadEnvironment("SKETCHFAB_LICENSE_FILTER", "cc0,cc-by,cc-by-sa"),
            Description = "CSV license filter"
        };

        Option<string> sortByOption = new("--sort-by")
        {
            DefaultValueFactory = _ => ReadEnvironment("SKETCHFAB_SORT_BY", "relevance"),
            Description = "Sort order: relevance, likeCount, viewCount, publishedAt"
        };

        Option<bool> excludePaidOption = new("--exclude-paid")
        {
            DefaultValueFactory = _ => ReadEnvironmentBool("SKETCHFAB_EXCLUDE_PAID", true),
            Description = "Exclude paid/exclusive models"
        };

        Option<bool> skipDuplicatesOption = new("--skip-duplicates")
        {
            DefaultValueFactory = _ => true,
            Description = "Skip candidates already downloaded"
        };

        Option<int> maxConcurrentOption = new("--max-concurrent")
        {
            DefaultValueFactory = _ => ReadEnvironmentInt("SKETCHFAB_CONCURRENT_REQUESTS", 1),
            Description = "Maximum concurrent downloads"
        };

        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("download-batch-sketchfab", "Search and batch download Sketchfab candidates.");
        command.Add(queryArg);
        command.Add(outputDirOption);
        command.Add(limitOption);
        command.Add(maxPolyOption);
        command.Add(minPolyOption);
        command.Add(licenseOption);
        command.Add(sortByOption);
        command.Add(excludePaidOption);
        command.Add(skipDuplicatesOption);
        command.Add(maxConcurrentOption);
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string query = parseResult.GetRequiredValue(queryArg);
            string outputDir = parseResult.GetValue(outputDirOption) ?? ReadEnvironment("ASSET_DOWNLOAD_DIR", "packs/warfare-starwars/assets/raw");
            int limit = parseResult.GetValue(limitOption);
            int maxPoly = parseResult.GetValue(maxPolyOption);
            int? minPoly = parseResult.GetValue(minPolyOption);
            string license = parseResult.GetValue(licenseOption) ?? "cc0,cc-by,cc-by-sa";
            string sortBy = parseResult.GetValue(sortByOption) ?? "relevance";
            bool excludePaid = parseResult.GetValue(excludePaidOption);
            bool skipDuplicates = parseResult.GetValue(skipDuplicatesOption);
            int maxConcurrent = parseResult.GetValue(maxConcurrentOption);
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            SketchfabClient client = serviceProvider.GetRequiredService<SketchfabClient>();
            AssetDownloader downloader = new(client, new AssetDownloaderOptions
            {
                DefaultLicenses = license,
                MaxPolyDefault = maxPoly,
                OutputDirectory = outputDir
            });

            try
            {
                System.Collections.Generic.IReadOnlyList<Sketchfab.AssetCandidate> candidates = await downloader.SearchCandidatesAsync(
                    query,
                    new AssetSearchCriteria
                    {
                        AllowedLicenses = license,
                        MaxPolyCount = maxPoly,
                        MinPolyCount = minPoly,
                        SortBy = sortBy,
                        ExcludePaid = excludePaid,
                        MaxCandidates = limit
                    },
                    ct);

                if (candidates.Count == 0)
                {
                    WriteError("download-batch-sketchfab", $"No candidates matched query '{query}'.", outputFormat);
                    return;
                }

                System.Collections.Generic.IReadOnlyList<Sketchfab.AssetCandidate> filteredCandidates = candidates;
                int duplicateCount = 0;
                if (skipDuplicates)
                {
                    System.Collections.Generic.IReadOnlyList<Sketchfab.AssetCandidate> deduped = downloader.DeduplicateCandidates(candidates, outputDir);
                    duplicateCount = candidates.Count - deduped.Count;
                    filteredCandidates = deduped;
                }

                if (filteredCandidates.Count == 0)
                {
                    WriteError("download-batch-sketchfab", "All candidates were skipped as duplicates.", outputFormat);
                    return;
                }

                System.Collections.Generic.List<Sketchfab.AssetCandidate> batchCandidates = filteredCandidates.Take(limit).ToList();
                System.Collections.Generic.List<BatchDownloadResult> results = (await downloader.DownloadBatchAsync(
                    batchCandidates,
                    outputDir,
                    maxConcurrent,
                    ct: ct)).ToList();

                int succeeded = results.Count(r => r.Success);
                int failed = results.Count - succeeded;
                long totalBytes = results.Where(r => r.Success && r.DownloadResult is not null).Sum(r => r.DownloadResult!.FileSizeBytes);
                long elapsedMs = results.Sum(r => r.DownloadResult?.DurationMs ?? 0);
                double throughputMpbs = elapsedMs > 0
                    ? ((totalBytes * 8d / 1_000_000d) / (elapsedMs / 1000d))
                    : 0;
                SketchfabRateLimitState? state = client.GetRateLimitState();

                if (!IsJsonOutput(outputFormat))
                {
                    AnsiConsole.MarkupLine($"[green]✓[/] Batch complete: {succeeded}/{batchCandidates.Count} succeeded, {failed} failed.");
                    if (state is not null)
                    {
                        AnsiConsole.MarkupLine($"Rate limit remaining: [yellow]{state.Remaining}[/], resets in [yellow]{state.ResetInSeconds}[/] seconds.");
                    }

                    return;
                }

                WriteJson(new
                {
                    success = true,
                    command = "download-batch-sketchfab",
                    query,
                    search_results = candidates.Count,
                    candidates_unique = filteredCandidates.Count,
                    candidates_downloaded = batchCandidates.Count,
                    candidates_duplicates = duplicateCount,
                    results = results.Select(r => new
                    {
                        model_id = r.Candidate.ModelId,
                        name = r.Candidate.Name,
                        status = r.Success ? "completed" : "failed",
                        asset_id = r.DownloadResult?.AssetId,
                        file_path = r.DownloadResult?.FilePath,
                        manifest_path = r.DownloadResult?.ManifestPath,
                        metadata_path = r.DownloadResult?.MetadataPath,
                        file_size_bytes = r.DownloadResult?.FileSizeBytes,
                        sha256 = r.DownloadResult?.Sha256,
                        duration_ms = r.DownloadResult?.DurationMs,
                        error = r.ErrorMessage
                    }),
                    failed,
                    summary = new
                    {
                        total_duration_ms = elapsedMs,
                        total_downloaded_bytes = totalBytes,
                        average_speed_mbps = throughputMpbs,
                        rate_limit_remaining = state?.Remaining
                    }
                });
            }
            catch (SketchfabApiException ex)
            {
                WriteError("download-batch-sketchfab", $"Sketchfab API error: {ex.Message}", outputFormat);
            }
            catch (Exception ex)
            {
                WriteError("download-batch-sketchfab", $"Batch download failed: {ex.Message}", outputFormat);
            }
        });

        return command;
    }

    private static Command CreateValidateSketchfabTokenCommand(IServiceProvider serviceProvider)
    {
        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("validate-sketchfab-token", "Validate a Sketchfab token and show quota info.");
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";

            ISketchfabAdapter adapter = serviceProvider.GetRequiredService<ISketchfabAdapter>();
            try
            {
                SketchfabTokenValidation validation = await adapter.ValidateTokenAsync(ct);

                string[] recommendations =
                [
                    "Use off-peak hours for batch downloads",
                    "Cache search results before repeated queries",
                    "Monitor quota with assetctl sketchfab-quota"
                ];

                if (!IsJsonOutput(outputFormat))
                {
                    if (validation.IsValid)
                    {
                        AnsiConsole.MarkupLine("[green]✓[/] Token is valid.");
                        AnsiConsole.MarkupLine($"  Plan: {Markup.Escape(validation.Plan ?? "unknown")}");
                        AnsiConsole.MarkupLine($"  Remaining quota: {validation.RemainingQuota}");
                        AnsiConsole.MarkupLine($"  Daily limit: {validation.DailyLimit}");
                        return;
                    }

                    AnsiConsole.MarkupLine("[yellow]ℹ[/] Token is not valid for requests.");
                    return;
                }

                WriteJson(new
                {
                    valid = validation.IsValid,
                    plan = validation.Plan,
                    remaining_quota = validation.RemainingQuota,
                    daily_limit = validation.DailyLimit,
                    token_scope = "Read",
                    recommendations
                });
            }
            catch (Exception ex)
            {
                WriteError("validate-sketchfab-token", $"Token validation failed: {ex.Message}", outputFormat);
            }
        });

        return command;
    }

    private static Command CreateSketchfabQuotaCommand(IServiceProvider serviceProvider)
    {
        Option<string> formatOption = new("--format")
        {
            DefaultValueFactory = _ => "text",
            Description = "Output format: text | json"
        };

        Command command = new("sketchfab-quota", "Show current Sketchfab API quota state.");
        command.Add(formatOption);

        command.SetAction(async (ParseResult parseResult, CancellationToken ct) =>
        {
            ct.ThrowIfCancellationRequested();
            string outputFormat = parseResult.GetValue(formatOption) ?? "text";
            ISketchfabAdapter adapter = serviceProvider.GetRequiredService<ISketchfabAdapter>();
            SketchfabRateLimitState? state;
            try
            {
                state = await adapter.GetQuotaAsync(ct);
            }
            catch (OperationCanceledException)
            {
                throw;
            }
            catch (Exception)
            {
                state = serviceProvider.GetRequiredService<SketchfabClient>().GetRateLimitState();
            }

            if (state is null)
            {
                WriteError("sketchfab-quota", "No rate limit state available. Run a Sketchfab command first.", outputFormat);
                return;
            }

            if (!IsJsonOutput(outputFormat))
            {
                AnsiConsole.MarkupLine("[bold]Sketchfab Quota[/]");
                AnsiConsole.MarkupLine($"Remaining: [green]{state.Remaining}[/], reset in [yellow]{state.ResetInSeconds}[/] seconds.");
                AnsiConsole.MarkupLine($"Reset at: [yellow]{state.ResetAtUtc:o}[/]");
                return;
            }

            WriteJson(new
            {
                remaining = state.Remaining,
                reset_at = state.ResetAtUtc.ToString("O", CultureInfo.InvariantCulture),
                reset_in_seconds = state.ResetInSeconds,
                last_checked_at = state.LastCheckedUtc.ToString("O", CultureInfo.InvariantCulture)
            });
        });

        return command;
    }

    private static bool IsJsonOutput(string outputFormat)
    {
        return string.Equals(outputFormat, "json", StringComparison.OrdinalIgnoreCase);
    }

    private static void WriteError(string command, string message, string outputFormat)
    {
        if (IsJsonOutput(outputFormat))
        {
            WriteJson(new
            {
                success = false,
                command,
                message
            });
            return;
        }

        AnsiConsole.MarkupLine($"[red]✗[/] {Markup.Escape(command)}: {Markup.Escape(message)}");
    }

    private static void WriteJson(object payload)
    {
        Console.WriteLine(JsonSerializer.Serialize(payload, JsonOptions));
    }

    private static bool TryParseCandidateRef(string candidateRef, out string source, out string externalId, out string parseError)
    {
        source = string.Empty;
        externalId = string.Empty;
        parseError = string.Empty;

        if (string.IsNullOrWhiteSpace(candidateRef))
        {
            parseError = "candidate reference cannot be empty; expected <source>:<externalId>";
            return false;
        }

        int separatorIndex = candidateRef.IndexOf(':');
        if (separatorIndex <= 0 || separatorIndex == candidateRef.Length - 1)
        {
            parseError = "candidate reference must be in format <source>:<externalId>";
            return false;
        }

        string left = candidateRef.Substring(0, separatorIndex).Trim();
        string right = candidateRef[(separatorIndex + 1)..].Trim();

        if (string.IsNullOrWhiteSpace(left) || string.IsNullOrWhiteSpace(right))
        {
            parseError = "candidate reference must be in format <source>:<externalId>";
            return false;
        }

        source = left;
        externalId = right;
        return true;
    }

    private static bool TryParseModelRef(string modelRef, out string source, out string modelId, out string parseError)
    {
        if (!TryParseCandidateRef(modelRef, out source, out modelId, out parseError))
        {
            return false;
        }

        return true;
    }

    private static string ReadEnvironment(string name, string defaultValue)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        return string.IsNullOrWhiteSpace(value) ? defaultValue : value;
    }

    private static int ReadEnvironmentInt(string name, int defaultValue)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        return int.TryParse(value, NumberStyles.Integer, CultureInfo.InvariantCulture, out int parsed)
            ? Math.Max(0, parsed)
            : defaultValue;
    }

    private static bool ReadEnvironmentBool(string name, bool defaultValue)
    {
        string? value = Environment.GetEnvironmentVariable(name);
        return bool.TryParse(value, out bool parsed) ? parsed : defaultValue;
    }

    private static void PatchManifestIp(DownloadAssetResult result, string ipStatus)
    {
        if (string.IsNullOrWhiteSpace(result.ManifestPath) || !File.Exists(result.ManifestPath))
        {
            return;
        }

        try
        {
            string existingManifest = File.ReadAllText(result.ManifestPath);
            AssetManifest? manifest = JsonSerializer.Deserialize<AssetManifest>(existingManifest, JsonOptions);
            if (manifest is null)
            {
                return;
            }

            manifest.IpStatus = ipStatus;
            string updated = JsonSerializer.Serialize(manifest, JsonOptions);
            File.WriteAllText(result.ManifestPath, updated);
        }
        catch
        {
            // Best effort: keep command flow deterministic even if manifest post-processing fails.
        }
    }
}
