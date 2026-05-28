<#
.SYNOPSIS
Subagent dispatch script for completing DINOForge telemetry implementation.

.DESCRIPTION
This script documents the 5 remaining tasks to complete telemetry system:

1. Instrument key runtime paths (4 files, 5 sites)
2. Add F10 telemetry panel UI
3. Add CLI metrics command
4. Add JSON-RPC getMetrics method to GameBridgeServer
5. Persist metrics on shutdown

MetricsCollector.cs already exists and builds successfully.
See: docs/sessions/telemetry_subagent_dispatch_20260528.md for full details.

.PARAMETER Task
Which task to work on (1-5) or 'all' for summary.

.EXAMPLE
.\telemetry-dispatch-subagent.ps1 -Task all
.\telemetry-dispatch-subagent.ps1 -Task 1
#>

param([string]$Task = "all")

$dispatch_doc = "C:\Users\koosh\Dino\docs\sessions\telemetry_subagent_dispatch_20260528.md"

Write-Host "
═══════════════════════════════════════════════════════════════════════
 DINOForge Telemetry Implementation - Subagent Dispatch
═══════════════════════════════════════════════════════════════════════

MetricsCollector.cs Status: ✅ CREATED
  Location: src/Runtime/Telemetry/MetricsCollector.cs
  Features:
    - Thread-safe ConcurrentDictionary backend
    - IncrementCounter, RecordValue, RecordDuration methods
    - DumpMarkdown() and DumpJson() serialization
    - String interning for zero-alloc metric names
    - Best-effort exception handling

Remaining Tasks:
  1. Instrument key runtime paths (4 files)
  2. Add F10 telemetry panel UI
  3. Add CLI metrics command
  4. Add JSON-RPC getMetrics method
  5. Persist metrics snapshot on shutdown

For full implementation details, see:
  $dispatch_doc

═══════════════════════════════════════════════════════════════════════
" -ForegroundColor Cyan

switch ($Task.ToLower()) {
    "all" {
        Write-Host "TASK SUMMARY: " -ForegroundColor Yellow
        Write-Host "
Task 1: INSTRUMENT RUNTIME PATHS
  Files to modify:
    - src/Runtime/ModPlatform.cs
    - src/Runtime/Bridge/AssetSwapSystem.cs
    - src/Runtime/Bridge/PackStatInjector.cs
    - src/Runtime/UI/NativeMenuInjector.cs

  Metrics to record:
    - pack_load.duration_ms, pack_load.count_loaded, pack_load.count_failed
    - asset_swap.update_calls, asset_swap.world_entity_count
    - stat_inject.writes_total, stat_inject.units_processed, stat_inject.duration_ms
    - mods_button.inject_attempts, mods_button.inject_success

Task 2: F10 TELEMETRY PANEL
  File: src/Runtime/UI/ModPanel.cs (or create new)
  Add new 'Telemetry' tab with auto-refreshing metrics table

Task 3: CLI METRICS COMMAND
  Create: src/Tools/Cli/Commands/MetricsCommand.cs
  Implement: 'dinoforge metrics' with --format json|markdown|table

Task 4: JSON-RPC METHOD
  File: src/Runtime/Bridge/GameBridgeServer.cs
  Add: HandleGetMetrics() handler in DispatchMethod switch

Task 5: METRICS PERSISTENCE
  File: src/Runtime/Plugin.cs
  Add: Metrics snapshot save in RuntimeDriver.OnDestroy()

Build & Test:
  dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release
  dotnet test src/Tests/
  Commit with: git add -A && git commit -m 'feat(telemetry): ...'

Return Value:
  - Commit hash
  - List of instrumented sites
  - Build output summary
  - Confirmation: '✅ All telemetry implementation complete'
" -ForegroundColor White
    }

    "1" {
        Write-Host "TASK 1: Instrument Runtime Paths" -ForegroundColor Yellow
        Write-Host @"

Sites to instrument:

1a. src/Runtime/ModPlatform.cs::LoadPacksImpl()
    - Add stopwatch at method entry
    - Record pack_load.duration_ms, count_loaded, count_failed

1b. src/Runtime/Bridge/AssetSwapSystem.cs::OnUpdate()
    - Increment asset_swap.update_calls
    - Record asset_swap.world_entity_count

1c. src/Runtime/Bridge/PackStatInjector.cs::Apply()
    - Record stat_inject.writes_total, units_processed, duration_ms

1d. src/Runtime/UI/NativeMenuInjector.cs::TryInjectMenuButton()
    - Increment mods_button.inject_attempts
    - Increment mods_button.inject_success

All methods:
  MetricsCollector.Instance.IncrementCounter(name)
  MetricsCollector.Instance.RecordValue(name, double)
  MetricsCollector.Instance.RecordDuration(name, TimeSpan)

See dispatch doc for exact code snippets.
"@
    }

    "2" {
        Write-Host "TASK 2: F10 Telemetry Panel" -ForegroundColor Yellow
        Write-Host @"

File: src/Runtime/UI/ModPanel.cs (or create new)

Add Telemetry tab that:
  - Displays metrics as UGUI grid/table
  - Refreshes every 2 seconds
  - Shows: | Metric | Value | Type | Samples |
  - Integrates with existing F10 panel navigation

Call:
  MetricsCollector.Instance.DumpMarkdown()

Render as UGUI table within overlay bounds.
"@
    }

    "3" {
        Write-Host "TASK 3: CLI Metrics Command" -ForegroundColor Yellow
        Write-Host @"

Create: src/Tools/Cli/Commands/MetricsCommand.cs

Features:
  - Command: 'dinoforge metrics'
  - Options: --format json|markdown|table (default: table)
  - Uses GameClient bridge to call getMetrics() JSON-RPC
  - Renders as Spectre.Console table

Register in Program.cs:
  commands.Add(new MetricsCommand());

See dispatch doc for full implementation.
"@
    }

    "4" {
        Write-Host "TASK 4: JSON-RPC getMetrics Method" -ForegroundColor Yellow
        Write-Host @"

File: src/Runtime/Bridge/GameBridgeServer.cs

Add to DispatchMethod() switch (around line 520):
  case 'getMetrics':
    return HandleGetMetrics();

Add handler:
  private JToken HandleGetMetrics()
  {
    return JToken.Parse(
      MetricsCollector.Instance.DumpJson());
  }

The CLI metrics command will call this via GameClient bridge.
"@
    }

    "5" {
        Write-Host "TASK 5: Metrics Persistence" -ForegroundColor Yellow
        Write-Host @"

File: src/Runtime/Plugin.cs (RuntimeDriver.OnDestroy)

Add near start of OnDestroy:
  string metricsJson = MetricsCollector.Instance.DumpJson();
  string path = Path.Combine(
    BepInEx.Paths.BepInExRootPath,
    'dinoforge-metrics-snapshot.json');
  File.WriteAllText(path, metricsJson);

Wrap in try/catch (best-effort, don't throw).

Output file: BepInEx/dinoforge-metrics-snapshot.json
"@
    }

    default {
        Write-Host "Unknown task: $Task. Use: 1, 2, 3, 4, 5, or 'all'" -ForegroundColor Red
        exit 1
    }
}

Write-Host "
═══════════════════════════════════════════════════════════════════════
Full dispatch documentation: $dispatch_doc
═══════════════════════════════════════════════════════════════════════
" -ForegroundColor Cyan
