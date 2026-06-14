#!/usr/bin/env bash

set -euo pipefail
#
# build-telemetry-view.sh — Generates a self-contained HTML telemetry viewer from DINOForge metrics
#
# Usage:
#   ./build-telemetry-view.sh [--metrics-path PATH] [--output-path PATH] [--watch] [--open]
#
# Examples:
#   ./build-telemetry-view.sh
#   ./build-telemetry-view.sh --metrics-path /custom/metrics.json --open
#   ./build-telemetry-view.sh --watch
#

set -e

METRICS_PATH=""
OUTPUT_PATH=""
WATCH=false
OPEN_BROWSER=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --metrics-path)
            METRICS_PATH="$2"
            shift 2
            ;;
        --output-path)
            OUTPUT_PATH="$2"
            shift 2
            ;;
        --watch)
            WATCH=true
            shift
            ;;
        --open)
            OPEN_BROWSER=true
            shift
            ;;
        *)
            echo "Unknown argument: $1"
            exit 1
            ;;
    esac
done

# Determine metrics path (default to game install)
if [ -z "$METRICS_PATH" ]; then
    if [ -n "$DINO_GAME_PATH" ]; then
        METRICS_PATH="$DINO_GAME_PATH/BepInEx/dinoforge-metrics-snapshot.json"
    else
        METRICS_PATH="/mnt/g/SteamLibrary/steamapps/common/Diplomacy is Not an Option/BepInEx/dinoforge-metrics-snapshot.json"
    fi
fi

# Determine output path
if [ -z "$OUTPUT_PATH" ]; then
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    REPO_ROOT="$(dirname "$SCRIPT_DIR")"
    OUTPUT_PATH="$REPO_ROOT/docs/telemetry/snapshot.html"
fi

# Check dependencies
if ! command -v jq &> /dev/null; then
    echo "Error: jq is required. Install with: apt-get install jq" >&2
    exit 1
fi

function generate_html() {
    local json_path="$1"
    local html_path="$2"

    if [ ! -f "$json_path" ]; then
        echo "Error: Metrics file not found: $json_path" >&2
        return 1
    fi

    echo "Reading metrics from: $json_path"

    # Ensure output directory exists
    mkdir -p "$(dirname "$html_path")"

    # Read JSON and extract metrics
    local json_data
    json_data=$(cat "$json_path")

    local timestamp
    timestamp=$(echo "$json_data" | jq -r '.timestamp // "unknown"')

    # Extract counters
    local counters=()
    local counter_labels=()
    local counter_values=()

    while IFS= read -r name; do
        local value
        value=$(echo "$json_data" | jq -r ".metrics[\"$name\"].raw // 0")
        counter_labels+=("\"$(echo "$name" | sed 's/"/\\"/g')\"")
        counter_values+=("$value")
    done < <(echo "$json_data" | jq -r '.metrics | to_entries[] | select(.value.type == "Counter") | .key')

    # Extract gauges
    local gauges=()
    local gauge_labels=()
    local gauge_values=()

    while IFS= read -r name; do
        local value
        value=$(echo "$json_data" | jq -r ".metrics[\"$name\"].raw // 0")
        gauge_labels+=("\"$(echo "$name" | sed 's/"/\\"/g')\"")
        gauge_values+=("$value")
    done < <(echo "$json_data" | jq -r '.metrics | to_entries[] | select(.value.type == "Value") | .key')

    # Extract durations
    local duration_labels=()
    local duration_avg=()
    local duration_total=()

    while IFS= read -r name; do
        local avg total
        avg=$(echo "$json_data" | jq -r ".metrics[\"$name\"].raw.avg_ms // 0")
        total=$(echo "$json_data" | jq -r ".metrics[\"$name\"].raw.total_ms // 0")
        duration_labels+=("\"$(echo "$name" | sed 's/"/\\"/g')\"")
        duration_avg+=("$avg")
        duration_total+=("$total")
    done < <(echo "$json_data" | jq -r '.metrics | to_entries[] | select(.value.type == "Duration") | .key')

    # Build counter labels/values arrays (JSON)
    local counter_labels_json="[$(IFS=,; echo "${counter_labels[*]}")]"
    local counter_values_json="[$(IFS=,; echo "${counter_values[*]}")]"
    local gauge_labels_json="[$(IFS=,; echo "${gauge_labels[*]}")]"
    local gauge_values_json="[$(IFS=,; echo "${gauge_values[*]}")]"
    local duration_labels_json="[$(IFS=,; echo "${duration_labels[*]}")]"
    local duration_avg_json="[$(IFS=,; echo "${duration_avg[*]}")]"
    local duration_total_json="[$(IFS=,; echo "${duration_total[*]}")]"

    # Build metrics table
    local table_rows=""
    while IFS= read -r line; do
        local name type value samples
        name=$(echo "$line" | jq -r '.key')
        type=$(echo "$line" | jq -r '.value.type')
        value=$(echo "$line" | jq -r '.value.value')
        samples=$(echo "$line" | jq -r '.value.samples')

        # HTML escape
        name=$(echo "$name" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g')
        value=$(echo "$value" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/>/\&gt;/g; s/"/\&quot;/g')

        table_rows+="        <tr>
            <td>$name</td>
            <td>$value</td>
            <td>$type</td>
            <td>$samples</td>
        </tr>
"
    done < <(echo "$json_data" | jq -c '.metrics | to_entries[]')

    local metric_count
    metric_count=$(echo "$json_data" | jq '.metrics | length')

    local counter_count gauge_count duration_count
    counter_count=$(echo "${#counter_labels[@]}")
    gauge_count=$(echo "${#gauge_labels[@]}")
    duration_count=$(echo "${#duration_labels[@]}")

    # Generate HTML
    cat > "$html_path" << 'EOF'
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

        .type-counter {
            background: rgba(255, 107, 107, 0.3);
            color: #ff6b6b;
        }

        .type-value {
            background: rgba(76, 175, 80, 0.3);
            color: #4caf50;
        }

        .type-duration {
            background: rgba(255, 193, 7, 0.3);
            color: #ffc107;
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
            <div class="timestamp">Captured: TIMESTAMP_PLACEHOLDER</div>
        </header>

        <div class="grid">
            <!-- Stats Overview -->
            <div class="card full-width">
                <h2>📊 Overview</h2>
                <div class="stats-grid">
                    <div class="stat-box">
                        <div class="stat-label">Total Metrics</div>
                        <div class="stat-value">METRIC_COUNT_PLACEHOLDER</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Counters</div>
                        <div class="stat-value">COUNTER_COUNT_PLACEHOLDER</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Gauges</div>
                        <div class="stat-value">GAUGE_COUNT_PLACEHOLDER</div>
                    </div>
                    <div class="stat-box">
                        <div class="stat-label">Durations</div>
                        <div class="stat-value">DURATION_COUNT_PLACEHOLDER</div>
                    </div>
                </div>
            </div>

            <!-- Counter Pie Chart -->
            COUNTER_CHART_PLACEHOLDER

            <!-- Gauge Bar Chart -->
            GAUGE_CHART_PLACEHOLDER

            <!-- Duration Line Chart -->
            DURATION_CHART_PLACEHOLDER

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
                        TABLE_ROWS_PLACEHOLDER
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

        CHARTS_SCRIPT_PLACEHOLDER
    </script>
</body>
</html>
EOF

    # Replace placeholders
    sed -i.bak "s|TIMESTAMP_PLACEHOLDER|$timestamp|g" "$html_path"
    sed -i.bak "s|METRIC_COUNT_PLACEHOLDER|$metric_count|g" "$html_path"
    sed -i.bak "s|COUNTER_COUNT_PLACEHOLDER|$counter_count|g" "$html_path"
    sed -i.bak "s|GAUGE_COUNT_PLACEHOLDER|$gauge_count|g" "$html_path"
    sed -i.bak "s|DURATION_COUNT_PLACEHOLDER|$duration_count|g" "$html_path"
    sed -i.bak "s|TABLE_ROWS_PLACEHOLDER|$table_rows|g" "$html_path"
    rm -f "${html_path}.bak"

    # Build and replace charts section
    local charts_script=""

    if [ $counter_count -gt 0 ]; then
        charts_script+="
        // Counter Doughnut Chart
        new Chart(document.getElementById('counterChart'), {
            type: 'doughnut',
            data: {
                labels: $counter_labels_json,
                datasets: [{
                    data: $counter_values_json,
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
"
    fi

    if [ $gauge_count -gt 0 ]; then
        charts_script+="
        // Gauge Bar Chart
        new Chart(document.getElementById('gaugeChart'), {
            type: 'bar',
            data: {
                labels: $gauge_labels_json,
                datasets: [{
                    label: 'Value',
                    data: $gauge_values_json,
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
"
    fi

    if [ $duration_count -gt 0 ]; then
        charts_script+="
        // Duration Line Chart
        new Chart(document.getElementById('durationChart'), {
            type: 'line',
            data: {
                labels: $duration_labels_json,
                datasets: [
                    {
                        label: 'Average (ms)',
                        data: $duration_avg_json,
                        borderColor: '#ffc107',
                        backgroundColor: 'rgba(255, 193, 7, 0.1)',
                        fill: false,
                        tension: 0.3,
                        pointRadius: 4,
                        pointHoverRadius: 6
                    },
                    {
                        label: 'Total (ms)',
                        data: $duration_total_json,
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
"
    fi

    sed -i.bak "s|CHARTS_SCRIPT_PLACEHOLDER|$charts_script|g" "$html_path"
    rm -f "${html_path}.bak"

    # Build counter chart card if counters exist
    local counter_card=""
    if [ $counter_count -gt 0 ]; then
        counter_card='
            <div class="card">
                <h2>Counters</h2>
                <div class="chart-container">
                    <canvas id="counterChart"></canvas>
                </div>
            </div>
'
    fi
    sed -i.bak "s|COUNTER_CHART_PLACEHOLDER|$counter_card|g" "$html_path"
    rm -f "${html_path}.bak"

    # Build gauge chart card if gauges exist
    local gauge_card=""
    if [ $gauge_count -gt 0 ]; then
        gauge_card='
            <div class="card">
                <h2>Gauges</h2>
                <div class="chart-container">
                    <canvas id="gaugeChart"></canvas>
                </div>
            </div>
'
    fi
    sed -i.bak "s|GAUGE_CHART_PLACEHOLDER|$gauge_card|g" "$html_path"
    rm -f "${html_path}.bak"

    # Build duration chart card if durations exist
    local duration_card=""
    if [ $duration_count -gt 0 ]; then
        duration_card='
            <div class="card">
                <h2>Durations</h2>
                <div class="chart-container">
                    <canvas id="durationChart"></canvas>
                </div>
            </div>
'
    fi
    sed -i.bak "s|DURATION_CHART_PLACEHOLDER|$duration_card|g" "$html_path"
    rm -f "${html_path}.bak"

    echo "✓ Generated: $html_path"
}

# Main execution
generate_html "$METRICS_PATH" "$OUTPUT_PATH"

if [ "$WATCH" = true ]; then
    echo "Watching for changes... Press Ctrl+C to stop"
    if command -v inotifywait &> /dev/null; then
        while true; do
            inotifywait -e modify "$METRICS_PATH" 2>/dev/null || true
            echo "Metrics file changed, regenerating..."
            generate_html "$METRICS_PATH" "$OUTPUT_PATH" || true
        done
    else
        echo "Warning: inotifywait not available. Install with: apt-get install inotify-tools"
        exit 1
    fi
fi

if [ "$OPEN_BROWSER" = true ]; then
    echo "Opening in browser..."
    if command -v xdg-open &> /dev/null; then
        xdg-open "$OUTPUT_PATH"
    elif command -v open &> /dev/null; then
        open "$OUTPUT_PATH"
    else
        echo "Warning: Could not find a way to open browser. File is at: $OUTPUT_PATH"
    fi
fi
