<#
.SYNOPSIS
    Generates a self-contained HTML telemetry viewer from a DINOForge metrics snapshot.

.DESCRIPTION
    Reads a metrics snapshot JSON file (from MetricsCollector.DumpJson()) and generates
    an interactive HTML dashboard with Chart.js visualizations.

.PARAMETER MetricsPath
    Path to the metrics snapshot JSON file.
    Default: Game install path BepInEx/dinoforge-metrics-snapshot.json

.PARAMETER OutputPath
    Path for the generated HTML file.
    Default: docs/telemetry/snapshot.html (relative to repo root)

.PARAMETER OpenBrowser
    If specified, opens the generated HTML in the default browser.

.EXAMPLE
    .\build-telemetry-view.ps1
    .\build-telemetry-view.ps1 -MetricsPath "C:\metrics.json" -OpenBrowser
#>

param(
    [string]$MetricsPath = "",
    [string]$OutputPath = "",
    [switch]$OpenBrowser
)

# Determine game install path
if (-not $MetricsPath) {
    $gameRoot = $env:DINO_GAME_PATH
    if (-not $gameRoot) {
        $gameRoot = "G:\SteamLibrary\steamapps\common\Diplomacy is Not an Option"
    }
    $MetricsPath = Join-Path $gameRoot "BepInEx\dinoforge-metrics-snapshot.json"
}

# Determine output path
if (-not $OutputPath) {
    $repoRoot = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
    $OutputPath = Join-Path $repoRoot "docs\telemetry\snapshot.html"
}

Write-Host "Reading metrics from: $MetricsPath" -ForegroundColor Cyan

if (-not (Test-Path $MetricsPath)) {
    Write-Error "Metrics file not found: $MetricsPath"
    exit 1
}

$json = Get-Content $MetricsPath -Raw | ConvertFrom-Json
Write-Host "Generating HTML to: $OutputPath" -ForegroundColor Cyan

$outputDir = Split-Path $OutputPath
if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir -Force | Out-Null
}

# Extract data from metrics
$counters = @()
$gauges = @()
$durations = @()
$timestamp = $json.timestamp

foreach ($name in $json.metrics.PSObject.Properties.Name) {
    $m = $json.metrics.$name
    $type = $m.type
    $value = $m.raw

    if ($type -eq "Counter") {
        $counters += @{ name = $name; value = $value }
    }
    elseif ($type -eq "Value") {
        $gauges += @{ name = $name; value = $value }
    }
    elseif ($type -eq "Duration") {
        $durations += @{ name = $name; total = $value.total_ms; avg = $value.avg_ms; samples = $m.samples }
    }
}

# Build counter labels/values arrays (JSON)
$counterLabels = $counters | Select-Object -ExpandProperty name | ConvertTo-Json -AsArray -Compress
$counterData = $counters | Select-Object -ExpandProperty value | ConvertTo-Json -AsArray -Compress

$gaugeLabels = $gauges | Select-Object -ExpandProperty name | ConvertTo-Json -AsArray -Compress
$gaugeData = $gauges | Select-Object -ExpandProperty value | ConvertTo-Json -AsArray -Compress

$durationLabels = $durations | Select-Object -ExpandProperty name | ConvertTo-Json -AsArray -Compress
$durationAvgData = $durations | Select-Object -ExpandProperty avg | ConvertTo-Json -AsArray -Compress
$durationTotalData = $durations | Select-Object -ExpandProperty total | ConvertTo-Json -AsArray -Compress

# Build metrics table rows
$tableRows = ""
foreach ($name in ($json.metrics.PSObject.Properties | Sort-Object -Property Name).Name) {
    $m = $json.metrics.$name
    $type = $m.type
    $value = $m.value
    $samples = $m.samples
    $name_escaped = [System.Web.HttpUtility]::HtmlEncode($name)
    $value_escaped = [System.Web.HttpUtility]::HtmlEncode($value)
    $tableRows += "        <tr>`r`n            <td>$name_escaped</td>`r`n            <td>$value_escaped</td>`r`n            <td>$type</td>`r`n            <td>$samples</td>`r`n        </tr>`r`n"
}

# Build chart sections conditionally
$counterChartHtml = ""
if ($counters.Count -gt 0) {
    $counterChartHtml = @"
            <div class="card">
                <h2>Counters</h2>
                <div class="chart-container">
                    <canvas id="counterChart"></canvas>
                </div>
            </div>

"@
}

$gaugeChartHtml = ""
if ($gauges.Count -gt 0) {
    $gaugeChartHtml = @"
            <div class="card">
                <h2>Gauges</h2>
                <div class="chart-container">
                    <canvas id="gaugeChart"></canvas>
                </div>
            </div>

"@
}

$durationChartHtml = ""
if ($durations.Count -gt 0) {
    $durationChartHtml = @"
            <div class="card">
                <h2>Durations</h2>
                <div class="chart-container">
                    <canvas id="durationChart"></canvas>
                </div>
            </div>

"@
}

# Build chart scripts
$chartsScript = ""

if ($counters.Count -gt 0) {
    $chartsScript += @"
        new Chart(document.getElementById('counterChart'), {
            type: 'doughnut',
            data: {
                labels: $counterLabels,
                datasets: [{
                    data: $counterData,
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

"@
}

if ($gauges.Count -gt 0) {
    $chartsScript += @"
        new Chart(document.getElementById('gaugeChart'), {
            type: 'bar',
            data: {
                labels: $gaugeLabels,
                datasets: [{
                    label: 'Value',
                    data: $gaugeData,
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

"@
}

if ($durations.Count -gt 0) {
    $chartsScript += @"
        new Chart(document.getElementById('durationChart'), {
            type: 'line',
            data: {
                labels: $durationLabels,
                datasets: [
                    {
                        label: 'Average (ms)',
                        data: $durationAvgData,
                        borderColor: '#ffc107',
                        backgroundColor: 'rgba(255, 193, 7, 0.1)',
                        fill: false,
                        tension: 0.3,
                        pointRadius: 4,
                        pointHoverRadius: 6
                    },
                    {
                        label: 'Total (ms)',
                        data: $durationTotalData,
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

"@
}

$metricCount = $json.metrics.PSObject.Properties.Count

# Generate final HTML
$html = @"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DINOForge Telemetry Snapshot</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js"></script>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #1e1e2e 0%, #2a2a3e 100%);
            color: #e0e0e0;
            padding: 20px;
            line-height: 1.6;
        }
        .container { max-width: 1400px; margin: 0 auto; }
        header {
            background: rgba(0, 0, 0, 0.4);
            border-left: 4px solid #00d4ff;
            padding: 20px;
            margin-bottom: 30px;
            border-radius: 4px;
            backdrop-filter: blur(10px);
        }
        h1 { font-size: 28px; margin-bottom: 8px; color: #00d4ff; }
        .timestamp { font-size: 12px; color: #888; font-family: 'Courier New', monospace; }
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
        .chart-container { position: relative; height: 300px; margin-bottom: 20px; }
        table { width: 100%; border-collapse: collapse; font-size: 13px; }
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
        tr:hover { background: rgba(0, 212, 255, 0.05); }
        .full-width { grid-column: 1 / -1; }
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
        .stat-label { font-size: 11px; color: #888; text-transform: uppercase; letter-spacing: 0.5px; }
        .stat-value { font-size: 20px; font-weight: bold; color: #00d4ff; font-family: 'Courier New', monospace; }
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🔬 DINOForge Telemetry Snapshot</h1>
            <div class="timestamp">Captured: $timestamp</div>
        </header>

        <div class="grid">
            <div class="card full-width">
                <h2>📊 Overview</h2>
                <div class="stats-grid">
                    <div class="stat-box">
                        <div class="stat-label">Total Metrics</div>
                        <div class="stat-value">$metricCount</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Counters</div>
                        <div class="stat-value">$($counters.Count)</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Gauges</div>
                        <div class="stat-value">$($gauges.Count)</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Durations</div>
                        <div class="stat-value">$($durations.Count)</div>
                    </div>
                </div>
            </div>

            $counterChartHtml$gaugeChartHtml$durationChartHtml

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
                        $tableRows
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

        $chartsScript
    </script>
</body>
</html>
"@

Set-Content -Path $OutputPath -Value $html -Encoding UTF8
Write-Host "✓ Generated: $OutputPath" -ForegroundColor Green

if ($OpenBrowser) {
    Write-Host "Opening in browser..." -ForegroundColor Cyan
    Start-Process -FilePath $OutputPath
}
