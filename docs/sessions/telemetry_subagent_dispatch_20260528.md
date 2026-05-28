# DINOForge Telemetry Implementation - Subagent Dispatch

**Dispatch Time**: 2026-05-28
**Base Commit**: 0cf468b4
**MetricsCollector Status**: ✅ DONE (created at `src/Runtime/Telemetry/MetricsCollector.cs`)

## Task Summary

Complete the telemetry system by instrumenting key runtime paths, adding F10 panel, CLI command, JSON-RPC method, and metrics persistence.

### 1. INSTRUMENT KEY RUNTIME PATHS

Files to modify:

#### 1a. `src/Runtime/ModPlatform.cs` - LoadPacksImpl() method (line ~555)

Add at the top of `LoadPacksImpl()`:
```csharp
// Add at line 556 after first check
var __metricsStartTime = System.Diagnostics.Stopwatch.StartNew();
```

At the end of the method (line ~640+), after logging success:
```csharp
// After L641: _log.LogInfo($"[ModPlatform] Successfully loaded {result.LoadedPacks.Count} pack(s).");
__metricsStartTime.Stop();
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordDuration(
    "pack_load.duration_ms", 
    __metricsStartTime.Elapsed);
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordValue(
    "pack_load.count_loaded", 
    result.LoadedPacks.Count);
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordValue(
    "pack_load.count_failed", 
    result.Errors.Count);
```

Imports to add at top:
```csharp
using DINOForge.Runtime.Telemetry;
```

#### 1b. `src/Runtime/Bridge/AssetSwapSystem.cs` - OnUpdate() method

Search for the OnUpdate method. Add at the start:
```csharp
// Increment update call counter
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.IncrementCounter("asset_swap.update_calls");
```

Then, where entity counts are computed (look for UniversalQuery or CalculateEntityCount calls):
```csharp
// Record entity count
int entityCount = 0;
try 
{ 
    entityCount = world.EntityManager.UniversalQuery.CalculateEntityCount(); 
}
catch { }
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordValue(
    "asset_swap.world_entity_count", 
    entityCount);
```

Imports:
```csharp
using DINOForge.Runtime.Telemetry;
```

#### 1c. `src/Runtime/Bridge/PackStatInjector.cs` - Apply() method

Find the Apply() static method. Add at method start:
```csharp
var __injectorStart = System.Diagnostics.Stopwatch.StartNew();
int __unitsProcessed = 0;
```

Where units are processed in the loop, increment counter:
```csharp
__unitsProcessed++;
```

Before return statement:
```csharp
__injectorStart.Stop();
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordValue(
    "stat_inject.writes_total", 
    injectedCount); // use existing injectedCount variable or count from loop
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordValue(
    "stat_inject.units_processed", 
    __unitsProcessed);
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.RecordDuration(
    "stat_inject.duration_ms", 
    __injectorStart.Elapsed);
```

Imports:
```csharp
using DINOForge.Runtime.Telemetry;
```

#### 1d. `src/Runtime/UI/NativeMenuInjector.cs` - TryInjectMenuButton() method

Find the TryInjectMenuButton method. Add at method start:
```csharp
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.IncrementCounter("mods_button.inject_attempts");
```

Before returning true (success path):
```csharp
DINOForge.Runtime.Telemetry.MetricsCollector.Instance.IncrementCounter("mods_button.inject_success");
```

Imports:
```csharp
using DINOForge.Runtime.Telemetry;
```

---

### 2. ADD F10 TELEMETRY PANEL

File: `src/Runtime/UI/ModPanel.cs` (or create new if doesn't exist)

Find the panel rendering code that handles tabs. Add a new "Telemetry" tab section:

```csharp
// Pseudo-code for where telemetry tab would be added
if (selectedTab == "Telemetry")
{
    var metricsMarkdown = DINOForge.Runtime.Telemetry.MetricsCollector.Instance.DumpMarkdown();
    // Parse markdown table and render as UGUI grid
    RenderMetricsTable(metricsMarkdown);
}
```

The metrics panel should:
- Display as a table: | Metric | Value | Type | Samples |
- Auto-refresh every 2 seconds (track time via Time.deltaTime)
- Fit within the F10 overlay bounds
- Use existing UGUI styling from other panels

Search for existing panel tabs to see the pattern and integrate telemetry alongside them.

---

### 3. ADD CLI COMMAND `dinoforge metrics`

Create new file: `src/Tools/Cli/Commands/MetricsCommand.cs`

```csharp
#nullable enable
using System;
using System.CommandLine;
using System.CommandLine.Invocation;
using System.Threading.Tasks;
using Spectre.Console;
using DINOForge.Bridge.Client;

namespace DINOForge.Tools.Cli.Commands
{
    internal class MetricsCommand : Command
    {
        public MetricsCommand() : base("metrics", "Display runtime metrics from the game")
        {
            var formatOption = new Option<string>(
                new[] { "--format", "-f" },
                () => "table",
                "Output format: table, json, or markdown");
            AddOption(formatOption);
            this.SetHandler(ExecuteAsync);
        }

        private async Task<int> ExecuteAsync(InvocationContext context)
        {
            var formatArg = context.ParseResult.GetValueForOption<string>("--format") ?? "table";

            try
            {
                using var client = new GameClient();
                var response = await client.InvokeAsync("getMetrics", null);

                if (response == null)
                {
                    AnsiConsole.MarkupLine("[red]Error:[/] Failed to retrieve metrics from game.");
                    return 1;
                }

                var metricsJson = response.ToString();

                switch (formatArg.ToLowerInvariant())
                {
                    case "json":
                        AnsiConsole.WriteLine(metricsJson);
                        break;
                    case "markdown":
                        // Parse JSON and render as markdown
                        var markdown = ParseMetricsAsMarkdown(metricsJson);
                        AnsiConsole.WriteLine(markdown);
                        break;
                    case "table":
                    default:
                        RenderMetricsTable(metricsJson);
                        break;
                }

                return 0;
            }
            catch (Exception ex)
            {
                AnsiConsole.MarkupLine($"[red]Error:[/] {ex.Message}");
                return 1;
            }
        }

        private void RenderMetricsTable(string metricsJson)
        {
            // Parse the JSON response and render as Spectre table
            // Expected format from MetricsCollector.DumpJson():
            // {
            //   "timestamp": "...",
            //   "metrics": {
            //     "metric_name": { "value": "...", "type": "...", "samples": ... }
            //   }
            // }

            var table = new Table();
            table.AddColumn("Metric");
            table.AddColumn("Value");
            table.AddColumn("Type");
            table.AddColumn("Samples");

            try
            {
                dynamic json = Newtonsoft.Json.JsonConvert.DeserializeObject(metricsJson);
                foreach (var metric in json["metrics"])
                {
                    table.AddRow(
                        metric.Name,
                        metric.Value.value?.ToString() ?? "—",
                        metric.Value.type?.ToString() ?? "—",
                        metric.Value.samples?.ToString() ?? "—");
                }
            }
            catch { }

            AnsiConsole.Write(table);
        }

        private string ParseMetricsAsMarkdown(string metricsJson)
        {
            // Convert JSON metrics to markdown table format
            // Similar to MetricsCollector.DumpMarkdown() output
            var sb = new System.Text.StringBuilder();
            sb.AppendLine("# DINOForge Metrics");
            sb.AppendLine();
            sb.AppendLine("| Metric | Value | Type | Samples |");
            sb.AppendLine("|--------|-------|------|---------|");

            try
            {
                dynamic json = Newtonsoft.Json.JsonConvert.DeserializeObject(metricsJson);
                foreach (var metric in json["metrics"])
                {
                    sb.AppendLine($"| {metric.Name} | {metric.Value.value} | {metric.Value.type} | {metric.Value.samples} |");
                }
            }
            catch { }

            return sb.ToString();
        }
    }
}
```

Register in Program.cs or wherever CLI commands are registered:
```csharp
// Add to command list
commands.Add(new MetricsCommand());
```

---

### 4. ADD JSON-RPC METHOD TO GameBridgeServer

File: `src/Runtime/Bridge/GameBridgeServer.cs`

In DispatchMethod switch statement (around line 520), add:
```csharp
case "getMetrics":
    return HandleGetMetrics();
```

Then add the handler method (with other handlers, around line 600+):
```csharp
/// <summary>
/// Handles the <c>getMetrics</c> request. Returns current runtime metrics as JSON.
/// </summary>
private JToken HandleGetMetrics()
{
    try
    {
        string metricsJson = DINOForge.Runtime.Telemetry.MetricsCollector.Instance.DumpJson();
        return JToken.Parse(metricsJson);
    }
    catch (Exception ex)
    {
        return new JObject
        {
            ["error"] = ex.Message
        };
    }
}
```

Imports (add at top if not present):
```csharp
using DINOForge.Runtime.Telemetry;
```

---

### 5. PERSIST METRICS ON SHUTDOWN

File: `src/Runtime/Plugin.cs` - RuntimeDriver.OnDestroy() method (around line 2191)

Add near the top of OnDestroy (before or after existing cleanup):
```csharp
// Persist metrics snapshot
try
{
    string metricsSnapshot = DINOForge.Runtime.Telemetry.MetricsCollector.Instance.DumpJson();
    string metricsPath = Path.Combine(BepInEx.Paths.BepInExRootPath, "dinoforge-metrics-snapshot.json");
    File.WriteAllText(metricsPath, metricsSnapshot);
    DebugLog.Write("Plugin", $"[RuntimeDriver] Metrics snapshot saved: {metricsPath}");
}
catch (Exception ex)
{
    DebugLog.Write("Plugin", $"[RuntimeDriver] Failed to save metrics snapshot: {ex.Message}");
    // Best-effort: don't rethrow
}
```

Imports (add at top if not present):
```csharp
using DINOForge.Runtime.Telemetry;
```

---

## BUILD & TEST CHECKLIST

After all changes:

```bash
# Build
dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release

# Verify no errors
# (should exit 0)

# Smoke test
dotnet test src/Tests/ --filter "Category=Integration"

# Deploy and verify
dotnet build -p:DeployToGame=true

# Test F10 panel displays metrics
# Test: dinoforge metrics command
# Verify: dinoforge-metrics-snapshot.json created on game shutdown
```

---

## Files to Create

- `src/Runtime/Telemetry/MetricsCollector.cs` ✅ DONE
- `src/Tools/Cli/Commands/MetricsCommand.cs` (NEW)

## Files to Modify

- `src/Runtime/ModPlatform.cs` (LoadPacksImpl instrumentation)
- `src/Runtime/Bridge/AssetSwapSystem.cs` (OnUpdate instrumentation)
- `src/Runtime/Bridge/PackStatInjector.cs` (Apply instrumentation)
- `src/Runtime/UI/NativeMenuInjector.cs` (TryInjectMenuButton instrumentation)
- `src/Runtime/UI/ModPanel.cs` (add Telemetry tab) [OR create new]
- `src/Runtime/Bridge/GameBridgeServer.cs` (add getMetrics handler)
- `src/Runtime/Plugin.cs` (add metrics persistence)
- `src/Tools/Cli/Program.cs` (register MetricsCommand)

---

## Commit Message Template

```
feat(telemetry): in-memory metrics collector + F10 telemetry tab

- Add lightweight telemetry instrumentation to key runtime paths:
  * ModPlatform.LoadPacksImpl: pack_load.* metrics
  * AssetSwapSystem.OnUpdate: asset_swap.* metrics
  * PackStatInjector.Apply: stat_inject.* metrics
  * NativeMenuInjector.TryInjectMenuButton: mods_button.* metrics
- Add F10 debug panel "Telemetry" tab with auto-refreshing metrics table
- Add 'dinoforge metrics' CLI command for remote metrics pull
- Add GameBridgeServer.HandleGetMetrics() JSON-RPC method
- Add metrics snapshot persistence to dinoforge-metrics-snapshot.json
- Zero-allocation metric names via string interning
- Thread-safe ConcurrentDictionary backend
- Best-effort exception handling (never throws on metric recording)
```

---

## Success Criteria

✅ MetricsCollector.cs created and builds
✅ All 4 instrumentation sites modified
✅ F10 Telemetry panel displays current metrics
✅ `dinoforge metrics` CLI command works
✅ GameBridgeServer.getMetrics() responds with JSON
✅ Metrics snapshot file created on shutdown
✅ `dotnet build` exits 0
✅ Smoke tests pass
✅ Commit succeeds

---

## Return Value Format (for orchestrator)

After completion, return:
1. Commit hash (from `git log -1 --oneline`)
2. List of instrumented sites
3. Build output excerpt (last 10 lines)
4. Summary: "✅ All telemetry implementation complete"
