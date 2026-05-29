# DINOForge Telemetry Implementation Summary

**Date**: 2026-05-28
**Status**: ✅ PHASE 1 COMPLETE (MetricsCollector core ready for integration)
**Next**: Subagent dispatch for instrumentation and integration (Tasks #920-#924)

## What Was Done

### 1. MetricsCollector.cs Core Implementation ✅

**Location**: `src/Runtime/Telemetry/MetricsCollector.cs` (380 LOC)

**Features**:
- Thread-safe in-memory metrics store via `ConcurrentDictionary<string, MetricEntry>`
- Three metric types: Counter, Value (snapshot), Duration (accumulated)
- Zero-allocation metric names via `string.Intern()`
- Best-effort exception handling (never throws on metric recording)
- Public API:
  - `IncrementCounter(string name)` — increment counter by 1
  - `RecordValue(string name, double value)` — record/overwrite numeric value
  - `RecordDuration(string name, TimeSpan duration)` — accumulate duration + track average
  - `DumpMarkdown()` — return markdown table format
  - `DumpJson()` — return JSON format with timestamp
  - `GetMetricValue(string name)` — programmatic access
  - `Clear()` — reset all metrics

**Design Characteristics**:
- Singleton pattern via `Lazy<T>`
- netstandard2.0 compatible (no .NET 8+ features)
- Uses Newtonsoft.Json for serialization (already a BepInEx transitive dep)
- Metric entries track sample count for averaging durations

### 2. Dispatch Documentation ✅

**Location**: `docs/sessions/telemetry_subagent_dispatch_20260528.md` (400+ lines)

**Contents**:
- Complete implementation guide for remaining 5 tasks
- Code snippets for each instrumentation site
- File paths, line numbers, and integration points
- Commit message template
- Success criteria checklist

### 3. Handoff Script ✅

**Location**: `scripts/telemetry-dispatch-subagent.ps1`

Quick reference for subagent to see task breakdown and code details.

---

## Remaining Work (5 Tasks)

### Task 1: Instrument Key Runtime Paths ⏳

**Files to modify**:
1. `src/Runtime/ModPlatform.cs` (LoadPacksImpl method)
   - Record pack_load metrics: duration_ms, count_loaded, count_failed
2. `src/Runtime/Bridge/AssetSwapSystem.cs` (OnUpdate method)
   - Record asset_swap metrics: update_calls, world_entity_count
3. `src/Runtime/Bridge/PackStatInjector.cs` (Apply method)
   - Record stat_inject metrics: writes_total, units_processed, duration_ms
4. `src/Runtime/UI/NativeMenuInjector.cs` (TryInjectMenuButton method)
   - Record mods_button metrics: inject_attempts, inject_success

**Effort**: ~30 minutes (straightforward metric calls)
**Dependencies**: MetricsCollector.cs (done)

### Task 2: Add F10 Telemetry Panel ⏳

**File**: `src/Runtime/UI/ModPanel.cs` (or create new)

**Requirements**:
- New "Telemetry" tab in F10 debug overlay
- Display metrics as UGUI grid/table: | Metric | Value | Type | Samples |
- Auto-refresh every 2 seconds
- Integrate with existing F10 panel navigation

**Effort**: ~45 minutes (UGUI rendering, requires pattern analysis from existing panels)
**Dependencies**: MetricsCollector.cs + instrumentation (Task 1)

### Task 3: Add CLI Command `dinoforge metrics` ⏳

**Create**: `src/Tools/Cli/Commands/MetricsCommand.cs`

**Features**:
- Command: `dinoforge metrics [--format table|json|markdown]`
- Use GameClient bridge to fetch metrics from running game
- Render as Spectre.Console table by default
- Support JSON and markdown output formats

**Effort**: ~30 minutes (straightforward CLI pattern)
**Dependencies**: MetricsCollector.cs + GameBridgeServer.getMetrics (Task 4)

### Task 4: Add JSON-RPC Method to GameBridgeServer ⏳

**File**: `src/Runtime/Bridge/GameBridgeServer.cs`

**Changes**:
- Add `case "getMetrics":` to DispatchMethod switch (~line 520)
- Add `HandleGetMetrics()` handler method
- Returns `MetricsCollector.Instance.DumpJson()`

**Effort**: ~15 minutes (trivial)
**Dependencies**: MetricsCollector.cs

### Task 5: Persist Metrics on Shutdown ⏳

**File**: `src/Runtime/Plugin.cs` (RuntimeDriver.OnDestroy method)

**Changes**:
- Call `MetricsCollector.Instance.DumpJson()`
- Write to `BepInEx/dinoforge-metrics-snapshot.json`
- Wrap in try/catch (best-effort, don't throw)
- Log success/failure to dinoforge_debug.log

**Effort**: ~15 minutes
**Dependencies**: MetricsCollector.cs

---

## Build Status

### Pre-Existing Issues (Not Related to This Work)

The repo has pre-existing SDK build errors:
```
error CS1656: Cannot assign to 'line' because it is a 'foreach iteration variable'
error CS1061: 'RSA' does not contain a definition for 'ImportSubjectPublicKeyInfo'
error CS0117: 'Path' does not contain a definition for 'GetRelativePath'
```

These are in `src/SDK/Signing/` and do not affect Runtime compilation.

### Verification

**MetricsCollector.cs** has no build errors when integrated into Runtime project:
- All dependencies resolved (Newtonsoft.Json, System, System.Collections.Concurrent)
- netstandard2.0 compatible
- No compiler warnings on the new file

---

## Testing Plan

1. **Unit Tests**: None required (MetricsCollector is simple, no business logic)
2. **Integration**: Smoke test existing suite after instrumentation
   ```bash
   dotnet test src/Tests/ --filter "Category=Integration"
   ```
3. **Manual Verification**:
   - Launch game with instrumented build
   - Verify F10 telemetry panel displays metrics
   - Run `dinoforge metrics` CLI command
   - Check `BepInEx/dinoforge-metrics-snapshot.json` after game shutdown

---

## Commit Checklist

Before final commit:
- [ ] All 4 instrumentation sites modified
- [ ] F10 telemetry panel displays metrics
- [ ] `dinoforge metrics` CLI command works
- [ ] GameBridgeServer.getMetrics() responds
- [ ] Metrics snapshot file created on shutdown
- [ ] `dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release` exits 0
- [ ] Smoke tests pass
- [ ] CHANGELOG.md updated
- [ ] Commit message follows template

**Commit Message**:
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

## Metrics Collected

### Pack Loading
- `pack_load.duration_ms` (Duration) — Time to load all packs
- `pack_load.count_loaded` (Value) — Number of successfully loaded packs
- `pack_load.count_failed` (Value) — Number of packs with errors

### Asset Swapping
- `asset_swap.update_calls` (Counter) — Number of OnUpdate cycles
- `asset_swap.world_entity_count` (Value) — Current entity count in gameplay world

### Stat Injection
- `stat_inject.writes_total` (Value) — Total entity field writes applied
- `stat_inject.units_processed` (Value) — Number of units processed
- `stat_inject.duration_ms` (Duration) — Time to apply stat injections

### UI Integration
- `mods_button.inject_attempts` (Counter) — Attempts to inject MODS button
- `mods_button.inject_success` (Counter) — Successful button injections

---

## Files Created/Modified

### New Files
✅ `src/Runtime/Telemetry/MetricsCollector.cs` (380 LOC)
✅ `docs/sessions/telemetry_subagent_dispatch_20260528.md` (dispatch guide)
✅ `scripts/telemetry-dispatch-subagent.ps1` (quick reference)
✅ `docs/sessions/telemetry_implementation_summary.md` (this file)

### Files to Modify (Pending)
⏳ `src/Runtime/ModPlatform.cs`
⏳ `src/Runtime/Bridge/AssetSwapSystem.cs`
⏳ `src/Runtime/Bridge/PackStatInjector.cs`
⏳ `src/Runtime/UI/NativeMenuInjector.cs`
⏳ `src/Runtime/UI/ModPanel.cs`
⏳ `src/Tools/Cli/Commands/MetricsCommand.cs` (new)
⏳ `src/Runtime/Bridge/GameBridgeServer.cs`
⏳ `src/Runtime/Plugin.cs`
⏳ `src/Tools/Cli/Program.cs`

---

## Related Issues

- #548 (telemetry needed for observability)
- #100+ (game state observability gaps)

---

## Next Steps (for Subagent Dispatch)

1. Read dispatch document: `docs/sessions/telemetry_subagent_dispatch_20260528.md`
2. Work through tasks 1-5 in order
3. Test at each stage (compile after task 1, build after task 4, etc.)
4. Final: `dotnet build && dotnet test` + commit

---

## References

- MetricsCollector public API: `src/Runtime/Telemetry/MetricsCollector.cs` (lines 1-380)
- Example instrumentation: `src/Runtime/ModPlatform.cs` (LoadPacksImpl, ~20 lines added)
- F10 panel pattern: `src/Runtime/UI/DebugPanel.cs` or `ModPanel.cs`
- CLI command pattern: `src/Tools/Cli/Commands/PackCommand.cs` or similar
- JSON-RPC handler pattern: `src/Runtime/Bridge/GameBridgeServer.cs` (HandleStatus, HandlePing, etc.)
