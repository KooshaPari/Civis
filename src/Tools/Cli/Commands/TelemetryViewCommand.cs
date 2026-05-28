#nullable enable
using System;
using System.CommandLine;
using System.CommandLine.Invocation;
using System.Diagnostics;
using System.IO;
using System.Text;
using System.Text.Json;
using Newtonsoft.Json.Linq;
using Spectre.Console;

namespace DINOForge.Tools.Cli.Commands;

/// <summary>
/// Generates an interactive HTML telemetry viewer from the metrics snapshot
/// and opens it in the default browser.
/// </summary>
internal static class TelemetryViewCommand
{
    /// <summary>
    /// Creates the <c>telemetry view</c> command.
    /// Usage: <c>dinoforge telemetry view [--metrics-path PATH] [--output-path PATH] [--no-open]</c>
    /// </summary>
    public static Command Create()
    {
        var command = new Command("view", "Generate and display an interactive telemetry HTML viewer");

        var metricsPathOpt = new Option<string>(
            new[] { "--metrics-path", "-m" },
            description: "Path to the metrics JSON snapshot file. Default: game install BepInEx/dinoforge-metrics-snapshot.json",
            getDefaultValue: () => ""
        );

        var outputPathOpt = new Option<string>(
            new[] { "--output-path", "-o" },
            description: "Path for the generated HTML file. Default: docs/telemetry/snapshot.html",
            getDefaultValue: () => ""
        );

        var noBrowserOpt = new Option<bool>(
            new[] { "--no-open" },
            description: "Generate HTML but don't open in browser"
        );

        command.Add(metricsPathOpt);
        command.Add(outputPathOpt);
        command.Add(noBrowserOpt);

        command.SetHandler(async (metricsPath, outputPath, noBrowser) =>
        {
            await ExecuteAsync(metricsPath, outputPath, noBrowser);
        }, metricsPathOpt, outputPathOpt, noBrowserOpt);

        return command;
    }

    private static async Task ExecuteAsync(string metricsPath, string outputPath, bool noBrowser)
    {
        try
        {
            // Determine metrics path
            if (string.IsNullOrWhiteSpace(metricsPath))
            {
                string gameRoot = Environment.GetEnvironmentVariable("DINO_GAME_PATH")
                    ?? "G:\\SteamLibrary\\steamapps\\common\\Diplomacy is Not an Option";

                metricsPath = Path.Combine(gameRoot, "BepInEx", "dinoforge-metrics-snapshot.json");
            }

            // Determine output path
            if (string.IsNullOrWhiteSpace(outputPath))
            {
                string repoRoot = FindRepoRoot();
                outputPath = Path.Combine(repoRoot, "docs", "telemetry", "snapshot.html");
            }

            // Verify metrics file exists
            if (!File.Exists(metricsPath))
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] Metrics file not found: {metricsPath}");
                AnsiConsole.MarkupLine("[dim]Hint: Make sure the game has run and metrics were captured.[/]");
                return;
            }

            AnsiConsole.MarkupLine($"[cyan]Reading metrics from:[/] {metricsPath}");

            // Read and parse metrics JSON
            string jsonContent = await File.ReadAllTextAsync(metricsPath).ConfigureAwait(false);
            JObject metricsObj = JObject.Parse(jsonContent);

            // Ensure output directory exists
            string outputDir = Path.GetDirectoryName(outputPath)!;
            if (!Directory.Exists(outputDir))
            {
                Directory.CreateDirectory(outputDir);
            }

            // Generate HTML
            string html = GenerateHtml(metricsObj);
            await File.WriteAllTextAsync(outputPath, html, Encoding.UTF8).ConfigureAwait(false);

            AnsiConsole.MarkupLine($"[green]✓ Generated:[/] {outputPath}");

            // Open in browser if requested
            if (!noBrowser)
            {
                AnsiConsole.MarkupLine("[cyan]Opening in browser...[/]");
                try
                {
                    var psi = new ProcessStartInfo
                    {
                        FileName = outputPath,
                        UseShellExecute = true
                    };
                    Process.Start(psi);
                }
                catch (Exception ex)
                {
                    AnsiConsole.MarkupLine($"[yellow]Warning:[/] Could not open browser: {ex.Message}");
                    AnsiConsole.MarkupLine($"[dim]File is available at: {outputPath}[/]");
                }
            }
        }
        catch (Exception ex)
        {
            AnsiConsole.MarkupLine($"[red]Error:[/] {ex.Message}");
            if (!string.IsNullOrEmpty(ex.StackTrace))
            {
                AnsiConsole.MarkupLine($"[dim]{ex.StackTrace}[/]");
            }
        }
    }

    private static string GenerateHtml(JObject metricsObj)
    {
        string timestamp = metricsObj.Value<string>("timestamp") ?? DateTime.UtcNow.ToString("O");

        // Extract metrics by type
        var counters = new List<(string Name, long Value)>(StringComparer.Ordinal.GetHashCode().GetType() == typeof(int) ? 10 : 10);
        var gauges = new List<(string Name, double Value)>(StringComparer.Ordinal.GetHashCode().GetType() == typeof(int) ? 10 : 10);
        var durations = new List<(string Name, double Avg, double Total, long Samples)>(StringComparer.Ordinal.GetHashCode().GetType() == typeof(int) ? 10 : 10);

        var metricsDict = metricsObj.Value<JObject>("metrics");
        if (metricsDict != null)
        {
            foreach (var kvp in metricsDict)
            {
                string name = kvp.Key;
                var metric = kvp.Value as JObject;
                if (metric == null) continue;

                string? type = metric.Value<string>("type");
                var raw = metric.Value<JToken>("raw");

                if (type == "Counter" && raw?.Type == JTokenType.Integer)
                {
                    counters.Add((name, raw.Value<long>()));
                }
                else if (type == "Value" && raw?.Type == JTokenType.Float || raw?.Type == JTokenType.Integer)
                {
                    gauges.Add((name, Convert.ToDouble(raw)));
                }
                else if (type == "Duration" && raw is JObject durObj)
                {
                    double avg = durObj.Value<double>("avg_ms");
                    double total = durObj.Value<double>("total_ms");
                    long samples = metric.Value<long>("samples");
                    durations.Add((name, avg, total, samples));
                }
            }
        }

        // Build Chart.js datasets
        var counterLabels = JsonSerializer.Serialize(counters.Select(c => c.Name).ToList());
        var counterData = JsonSerializer.Serialize(counters.Select(c => c.Value).ToList());

        var gaugeLabels = JsonSerializer.Serialize(gauges.Select(g => g.Name).ToList());
        var gaugeData = JsonSerializer.Serialize(gauges.Select(g => g.Value).ToList());

        var durationLabels = JsonSerializer.Serialize(durations.Select(d => d.Name).ToList());
        var durationAvgData = JsonSerializer.Serialize(durations.Select(d => d.Avg).ToList());
        var durationTotalData = JsonSerializer.Serialize(durations.Select(d => d.Total).ToList());

        // Build metrics table
        var tableRows = new StringBuilder();
        if (metricsDict != null)
        {
            foreach (var kvp in metricsDict.OrderBy(x => x.Key))
            {
                string name = System.Web.HttpUtility.HtmlEncode(kvp.Key);
                var metric = kvp.Value as JObject;
                if (metric == null) continue;

                string? value = metric.Value<string>("value");
                string? type = metric.Value<string>("type");
                long samples = metric.Value<long>("samples");

                if (value != null)
                {
                    value = System.Web.HttpUtility.HtmlEncode(value);
                    tableRows.AppendLine($"        <tr>");
                    tableRows.AppendLine($"            <td>{name}</td>");
                    tableRows.AppendLine($"            <td>{value}</td>");
                    tableRows.AppendLine($"            <td>{type}</td>");
                    tableRows.AppendLine($"            <td>{samples}</td>");
                    tableRows.AppendLine($"        </tr>");
                }
            }
        }

        // Generate chart sections (conditional)
        var counterChartHtml = counters.Count > 0
            ? """
            <div class="card">
                <h2>Counters</h2>
                <div class="chart-container">
                    <canvas id="counterChart"></canvas>
                </div>
            </div>

            """
            : string.Empty;

        var gaugeChartHtml = gauges.Count > 0
            ? """
            <div class="card">
                <h2>Gauges</h2>
                <div class="chart-container">
                    <canvas id="gaugeChart"></canvas>
                </div>
            </div>

            """
            : string.Empty;

        var durationChartHtml = durations.Count > 0
            ? """
            <div class="card">
                <h2>Durations</h2>
                <div class="chart-container">
                    <canvas id="durationChart"></canvas>
                </div>
            </div>

            """
            : string.Empty;

        // Generate chart scripts (conditional)
        var chartsScript = new StringBuilder();

        if (counters.Count > 0)
        {
            chartsScript.AppendLine("""
        // Counter Doughnut Chart
        new Chart(document.getElementById('counterChart'), {
            type: 'doughnut',
            data: {
                labels: """ + counterLabels + @""",
                datasets: [{
                    data: """ + counterData + @""",
                    backgroundColor: [
                        '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7',
                        '#dfe6e9', '#fd79a8', '#fdcb6e', '#6c5ce7', '#a29bfe'
                    ],
                    borderColor: 'rgba(255, 255, 255, 0.1)',
                    borderWidth: 2
                }]
            },
            options: chartConfig
        });

""");
        }

        if (gauges.Count > 0)
        {
            chartsScript.AppendLine("""
        // Gauge Bar Chart
        new Chart(document.getElementById('gaugeChart'), {
            type: 'bar',
            data: {
                labels: """ + gaugeLabels + @""",
                datasets: [{
                    label: 'Value',
                    data: """ + gaugeData + @""",
                    backgroundColor: '#4caf50',
                    borderColor: 'rgba(76, 175, 80, 0.5)',
                    borderWidth: 1
                }]
            },
            options: {
                ...chartConfig,
                indexAxis: 'y'
            }
        });

""");
        }

        if (durations.Count > 0)
        {
            chartsScript.AppendLine("""
        // Duration Line Chart
        new Chart(document.getElementById('durationChart'), {
            type: 'line',
            data: {
                labels: """ + durationLabels + @""",
                datasets: [
                    {
                        label: 'Average (ms)',
                        data: """ + durationAvgData + @""",
                        borderColor: '#ffc107',
                        backgroundColor: 'rgba(255, 193, 7, 0.1)',
                        fill: false,
                        tension: 0.3,
                        pointRadius: 4,
                        pointHoverRadius: 6
                    },
                    {
                        label: 'Total (ms)',
                        data: """ + durationTotalData + @""",
                        borderColor: '#ff6b6b',
                        backgroundColor: 'rgba(255, 107, 107, 0.1)',
                        fill: false,
                        tension: 0.3,
                        pointRadius: 4,
                        pointHoverRadius: 6
                    }
                ]
            },
            options: chartConfig
        });

""");
        }

        // Build final HTML
        return $$"""
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DINOForge Telemetry Snapshot</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js"></script>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #1e1e2e 0%, #2a2a3e 100%);
            color: #e0e0e0;
            padding: 20px;
            line-height: 1.6;
        }

        .container {
            max-width: 1400px;
            margin: 0 auto;
        }

        header {
            background: rgba(0, 0, 0, 0.4);
            border-left: 4px solid #00d4ff;
            padding: 20px;
            margin-bottom: 30px;
            border-radius: 4px;
            backdrop-filter: blur(10px);
        }

        h1 {
            font-size: 28px;
            margin-bottom: 8px;
            color: #00d4ff;
        }

        .timestamp {
            font-size: 12px;
            color: #888;
            font-family: 'Courier New', monospace;
        }

        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }

        .card {
            background: rgba(255, 255, 255, 0.05);
            border: 1px solid rgba(0, 212, 255, 0.2);
            border-radius: 8px;
            padding: 20px;
            backdrop-filter: blur(10px);
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
        }

        .card h2 {
            font-size: 16px;
            margin-bottom: 15px;
            color: #00d4ff;
            border-bottom: 1px solid rgba(0, 212, 255, 0.3);
            padding-bottom: 10px;
        }

        .chart-container {
            position: relative;
            height: 300px;
            margin-bottom: 20px;
        }

        table {
            width: 100%;
            border-collapse: collapse;
            font-size: 13px;
        }

        th {
            background: rgba(0, 212, 255, 0.1);
            color: #00d4ff;
            padding: 10px;
            text-align: left;
            font-weight: 600;
            border-bottom: 2px solid rgba(0, 212, 255, 0.3);
        }

        td {
            padding: 8px 10px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            font-family: 'Courier New', monospace;
            font-size: 12px;
        }

        tr:hover {
            background: rgba(0, 212, 255, 0.05);
        }

        .full-width {
            grid-column: 1 / -1;
        }

        footer {
            text-align: center;
            color: #666;
            font-size: 12px;
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid rgba(255, 255, 255, 0.05);
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 10px;
            margin-bottom: 15px;
        }

        .stat-box {
            background: rgba(0, 212, 255, 0.1);
            border-left: 3px solid #00d4ff;
            padding: 10px;
            border-radius: 4px;
        }

        .stat-label {
            font-size: 11px;
            color: #888;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }

        .stat-value {
            font-size: 20px;
            font-weight: bold;
            color: #00d4ff;
            font-family: 'Courier New', monospace;
        }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🔬 DINOForge Telemetry Snapshot</h1>
            <div class="timestamp">Captured: {{timestamp}}</div>
        </header>

        <div class="grid">
            <!-- Stats Overview -->
            <div class="card full-width">
                <h2>📊 Overview</h2>
                <div class="stats-grid">
                    <div class="stat-box">
                        <div class="stat-label">Total Metrics</div>
                        <div class="stat-value">{{metricsObj.Value<JObject>("metrics")?.Count ?? 0}}</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Counters</div>
                        <div class="stat-value">{{counters.Count}}</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Gauges</div>
                        <div class="stat-value">{{gauges.Count}}</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Durations</div>
                        <div class="stat-value">{{durations.Count}}</div>
                    </div>
                </div>
            </div>

            {{counterChartHtml}}{{gaugeChartHtml}}{{durationChartHtml}}

            <!-- Metrics Table -->
            <div class="card full-width">
                <h2>📋 All Metrics</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Metric Name</th>
                            <th>Value</th>
                            <th>Type</th>
                            <th>Samples</th>
                        </tr>
                    </thead>
                    <tbody>
                        {{tableRows}}
                    </tbody>
                </table>
            </div>
        </div>

        <footer>
            <p>Generated by DINOForge Telemetry Viewer | Chart.js v4.4.0</p>
        </footer>
    </div>

    <script>
        const chartConfig = {
            responsive: true,
            maintainAspectRatio: false,
            plugins: {
                legend: {
                    labels: {
                        color: '#e0e0e0',
                        font: { size: 12 }
                    }
                }
            },
            scales: {
                y: {
                    ticks: { color: '#888' },
                    grid: { color: 'rgba(255, 255, 255, 0.05)' }
                },
                x: {
                    ticks: { color: '#888' },
                    grid: { color: 'rgba(255, 255, 255, 0.05)' }
                }
            }
        };

        {{chartsScript}}
    </script>
</body>
</html>
""";
    }

    private static string FindRepoRoot()
    {
        string current = Directory.GetCurrentDirectory();
        while (current != null)
        {
            if (File.Exists(Path.Combine(current, "DINOForge.sln"))
                || File.Exists(Path.Combine(current, ".git")))
            {
                return current;
            }
            current = Directory.GetParent(current)?.FullName;
        }
        return Directory.GetCurrentDirectory();
    }
}
