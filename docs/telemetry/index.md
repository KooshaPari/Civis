# DINOForge Telemetry System

DINOForge includes a comprehensive runtime telemetry system to monitor performance, track operations, and diagnose issues. This guide explains how to use the metrics collection, export, and visualization features.

## Overview

The telemetry system consists of three layers:

| Layer | Purpose | Technology |
|-------|---------|-----------|
| **Runtime Collection** | Records metrics during gameplay | `MetricsCollector.cs` (in-game) |
| **Snapshot Export** | Captures metrics to JSON file | BepInEx plugin lifecycle |
| **Web Visualization** | Interactive dashboard with charts | Chart.js + HTML5 |

## Metric Types

The MetricsCollector tracks three types of metrics:

### Counters
Increment-only counters that accumulate events.

**Use for**: Call counts, operation events, total occurrences
**Example metrics**:
- `asset_swap.update_calls` — How many times AssetSwapSystem updated
- `stat_inject.writes_total` — Total stat modifier applications
- `pack.load_count` — Number of packs loaded

**Format in JSON**:
```json
{
  "asset_swap.update_calls": {
    "type": "Counter",
    "value": "42",
    "samples": 1,
    "raw": 42
  }
}
```

### Values (Gauges)
Numeric measurements that represent instantaneous state.

**Use for**: Quantities, counts, measurements at a point in time
**Example metrics**:
- `asset_swap.world_entity_count` — How many entities are in the ECS world
- `memory.current_mb` — Current memory usage
- `pack.active_count` — Number of currently active packs

**Format in JSON**:
```json
{
  "asset_swap.world_entity_count": {
    "type": "Value",
    "value": "49014.00",
    "samples": 1,
    "raw": 49014.0
  }
}
```

### Durations
Time measurements with accumulated samples and running averages.

**Use for**: Operation timings, performance measurements
**Example metrics**:
- `pack_load.total_ms` — How long pack loading takes
- `asset_swap.frame_time_ms` — Time spent in asset swaps per frame
- `registry.insert_ms` — Time to insert into registries

**Format in JSON**:
```json
{
  "pack_load.total_ms": {
    "type": "Duration",
    "value": "Σ 254.3ms, avg 127.2ms",
    "samples": 2,
    "raw": {
      "total_ms": 254.3,
      "avg_ms": 127.2
    }
  }
}
```

## Enabling Metrics

Metrics are collected automatically by the DINOForge runtime. No configuration is required.

### Where Metrics Are Recorded

The MetricsCollector instance is available at runtime:

```csharp
// From any DINOForge plugin or system
MetricsCollector.Instance.IncrementCounter("my_event.count");
MetricsCollector.Instance.RecordValue("my_state.current", value);
MetricsCollector.Instance.RecordDuration("my_operation.ms", timeSpan);
```

### Exporting Snapshots

When the game exits or the plugin is unloaded, DINOForge automatically exports a metrics snapshot:

**File location**: `BepInEx/dinoforge-metrics-snapshot.json`

This file contains the complete state of all metrics at the time of export.

## Viewing Metrics

### Terminal Display

View metrics in the terminal:

```bash
dinoforge metrics show
dinoforge metrics              # Default (same as show)
```

**Output**: Formatted table with metric names and values

**Export as JSON**:
```bash
dinoforge metrics show --format json
```

### Interactive HTML Dashboard

Generate and display an interactive web dashboard:

```bash
dinoforge telemetry view
```

**What it does**:
1. Reads the metrics snapshot from `BepInEx/dinoforge-metrics-snapshot.json`
2. Generates a self-contained HTML file at `docs/telemetry/snapshot.html`
3. Opens it in your default browser

**Options**:
```bash
dinoforge telemetry view --help

Usage:
  dinoforge telemetry view [OPTIONS]

Options:
  -m, --metrics-path <path>      Path to metrics JSON (default: game install)
  -o, --output-path <path>       Path for generated HTML (default: docs/telemetry/)
  --no-open                      Generate HTML but don't open browser
```

## Dashboard Features

The interactive dashboard includes:

### 📊 Overview Stats
- Total metrics count
- Breakdown by type (Counters, Gauges, Durations)

### 📈 Visualizations

**Counter Chart (Doughnut)**
- Shows relative magnitudes of all counter values
- Use to identify which operations are most frequent

**Gauge Chart (Horizontal Bar)**
- Displays current state values
- Use to see instantaneous measurements (entity counts, memory, etc.)

**Duration Chart (Line Graph)**
- Shows average and total time for durations
- Use to identify performance bottlenecks and trending

### 📋 Metrics Table
- Complete list of all metrics with values, types, and sample counts
- Sortable and searchable in most browsers (use Ctrl+F)

## PowerShell Scripts

For build/CI environments, use the PowerShell script directly:

```powershell
.\scripts\build-telemetry-view.ps1 -MetricsPath "path\to\metrics.json"
.\scripts\build-telemetry-view.ps1 -OpenBrowser
.\scripts\build-telemetry-view.ps1 -Watch  # Auto-regenerate on change
```

## Bash Scripts

For Linux/macOS or WSL environments:

```bash
./scripts/build-telemetry-view.sh --metrics-path /path/to/metrics.json
./scripts/build-telemetry-view.sh --open
./scripts/build-telemetry-view.sh --watch  # Requires inotifywait
```

## Common Workflows

### Debugging Performance Issues

1. **Launch game** with DINOForge loaded
2. **Reproduce the issue**
3. **Exit the game** (metrics are exported on shutdown)
4. **View the dashboard**:
   ```bash
   dinoforge telemetry view
   ```
5. **Look at Duration metrics** for slow operations
6. **Check Counter metrics** for unexpected call counts

### Comparing Test Runs

1. Run test scenario A, capture metrics to `test_a.json`
2. Run test scenario B, capture metrics to `test_b.json`
3. Generate separate dashboards:
   ```bash
   dinoforge telemetry view -m test_a.json -o dashboard_a.html --no-open
   dinoforge telemetry view -m test_b.json -o dashboard_b.html --no-open
   ```
4. **Open both in browser tabs** for side-by-side comparison

### Historical Tracking

If you save snapshots with timestamps:

```bash
# In a script or CI pipeline
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
Copy-Item "$gameDir\BepInEx\dinoforge-metrics-snapshot.json" `
  "metrics_history\snapshot_$timestamp.json"
```

Then generate a dashboard for each snapshot to track changes over time.

## Metric Naming Convention

Metrics follow a dot-separated naming scheme:

```
<domain>.<subsystem>.<metric_name>
```

**Examples**:
- `asset_swap.update_calls` — Asset swap domain, update subsystem
- `pack_load.total_ms` — Pack loading domain, load timing
- `stat_inject.writes_total` — Stat injection domain, total writes

This allows filtering and aggregation by domain in future tools.

## Adding New Metrics

To instrument new code paths:

```csharp
using DINOForge.Runtime.Telemetry;

// In a system or plugin
public void MyOperation()
{
    // Count an event
    MetricsCollector.Instance.IncrementCounter("my_feature.events");

    // Record a value
    MetricsCollector.Instance.RecordValue("my_feature.current_state", someValue);

    // Time an operation
    var sw = Stopwatch.StartNew();
    try
    {
        // Do work
    }
    finally
    {
        MetricsCollector.Instance.RecordDuration("my_feature.operation_ms", sw.Elapsed);
    }
}
```

**Best practices**:
- Use descriptive metric names
- Follow the `domain.subsystem.name` convention
- Record timing in milliseconds for durations
- Keep the MetricsCollector calls in `try`/`catch` blocks (they are best-effort)
- Don't record sensitive data (passwords, player IDs, etc.)

## Architecture

### MetricsCollector (Runtime)

Located: `src/Runtime/Telemetry/MetricsCollector.cs`

**Key features**:
- Thread-safe concurrent dictionary storage
- Zero-allocation hot paths via string interning
- Three metric types (Counter, Value, Duration)
- Markdown and JSON export formats

### Export on Shutdown

The DINOForge Runtime plugin exports metrics when:
- Plugin is unloaded
- Game process exits
- Player disables the mod

**File**: `BepInEx/dinoforge-metrics-snapshot.json`

### HTML Generation

Both PowerShell and C# implementations generate identical self-contained HTML with:
- Embedded Chart.js library (via CDN)
- Responsive dark-theme design
- Client-side rendering (no server required)
- No external dependencies

## Troubleshooting

### "Metrics file not found"
```
dinoforge telemetry view
Error: Metrics file not found: ...
```

**Solution**: Make sure:
1. The game has run at least once with DINOForge loaded
2. The game process exited cleanly (closed via menu, not killed)
3. The file exists at `BepInEx/dinoforge-metrics-snapshot.json`

### "No metrics recorded yet"
The dashboard appears but the metrics table is empty.

**Possible causes**:
1. The game launched but didn't run long enough for metrics
2. Metrics collection is disabled (check Runtime config)
3. All operations completed too quickly to record

**Solution**: Play for a few seconds, load some packs, then exit cleanly.

### HTML doesn't display charts
The HTML loads but charts are blank or missing.

**Possible causes**:
1. Chart.js CDN is unreachable (offline?)
2. Browser is too old (needs ES6 support)
3. Metrics snapshot had unusual data

**Solution**:
- Check internet connection
- Try a different browser
- Verify JSON file is valid: `dinoforge metrics show --format json`

## See Also

- [MetricsCollector.cs](../../src/Runtime/Telemetry/MetricsCollector.cs) — Source code
- [CLI Tools](../concepts/cli-tools.md) — DINOForge command reference
- [Runtime Architecture](../architecture/runtime.md) — Plugin lifecycle and initialization
