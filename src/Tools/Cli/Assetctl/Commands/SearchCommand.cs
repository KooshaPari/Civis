#nullable enable
using System;
using System.Globalization;
using System.Linq;
using System.CommandLine;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Assetctl.Commands;

/// <summary>
/// Search command - searches candidates from configured intake sources.
/// </summary>
internal static class SearchCommand
{
    public static Command Create()
    {
        Argument<string> queryArg = new("query") { Description = "Search query text" };
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
        Option<int?> minPolyOption = new("--min-poly") { Description = "Minimum polygon estimate" };
        Option<int?> maxPolyOption = new("--max-poly") { Description = "Maximum polygon estimate" };
        Option<string> formatOption = AssetctlOptions.FormatOption();
        Option<string> pipelineRootOption = AssetctlOptions.PipelineRootOption();

        Command command = new("search", "Search candidates from configured intake sources.");
        command.Add(queryArg);
        command.Add(sourceOption);
        command.Add(limitOption);
        command.Add(licenseOption);
        command.Add(minPolyOption);
        command.Add(maxPolyOption);
        command.Add(formatOption);
        command.Add(pipelineRootOption);

        command.SetAction((ParseResult parseResult, CancellationToken ct) =>
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

            if (!AssetctlOutput.IsJsonOutput(outputFormat))
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
                return Task.CompletedTask;
            }

            AssetctlOutput.WriteJson(new
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
            return Task.CompletedTask;
        });

        return command;
    }
}
