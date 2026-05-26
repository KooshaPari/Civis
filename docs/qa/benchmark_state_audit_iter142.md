# Benchmark State Audit — Iteration 142

**Date**: 2026-05-18  
**Auditor**: Claude Haiku (Agent, read-only)  
**Status**: FINDINGS DOCUMENTED

---

## Executive Summary

The DINOForge benchmark infrastructure is **partially broken due to path mismatch** between the benchmark project location and the CI workflow. The benchmark suite exists but is **currently not runnable via CI** because `.github/workflows/benchmarks.yml` references the wrong project path. Committed baseline snapshots are minimal (4 entries, 2.9KB). No BenchmarkDotNet.Artifacts directory leaked into git staging.

---

## (a) BenchmarkDotNet.Artifacts Directory Status

**Size**: Not present on disk  
**File Count**: 0 (never created locally)  
**Reason**: The benchmark project has not been executed successfully on this machine.

**Artifacts on Disk**:
- `src/Tests/Benchmarks/bin/Release/net8.0/` contains compiled BenchmarkDotNet assemblies (~3 DLLs)
- These are ignored by `.gitignore` (covered by `bin/` pattern)
- No BenchmarkDotNet.Artifacts output directory generated

**Iteration 141 Finding (Regex Escape Bug)**: Not applicable — there is no BenchmarkDotNet.Artifacts directory to leak. The iter-141 audit flagged a *hypothetical* regex escape bug in `.gitignore` for the pattern, but since the artifacts directory never runs, the bug has no visible impact.

---

## (b) Committed Benchmark Snapshots in `docs/benchmarks/`

**Files**:
- `baseline.json` — 6 lines, 317 bytes (created 2026-03-29)
- `index.md` — 91 lines, 2,535 bytes (created 2026-03-29)

**Baseline Content** (4 entries):
```json
{
  "DINOForge.Tests.Benchmarks.ContentLoaderBenchmarks.LoadSimplePack": 50000000,
  "DINOForge.Tests.Benchmarks.ContentLoaderBenchmarks.LoadComplexPack": 150000000,
  "DINOForge.Tests.Benchmarks.RegistryBenchmarks.QueryUnits": 5000000,
  "DINOForge.Tests.Benchmarks.RegistryBenchmarks.RegisterBulkUnits": 10000000
}
```

**Status**: Baselines are **HARDCODED PLACEHOLDER VALUES** (all round multiples of 10M nanoseconds). These are NOT real measurements — they were set as target values before actual BenchmarkDotNet runs. The `index.md` document reflects this: it shows a VueJS template expecting `latest.json` to arrive after a CI run, but currently no `latest.json` exists in the repo.

**Last Updated**: 2026-03-29 (before MCP bridge work, 7 weeks ago).

---

## (c) .gitignore Pattern Analysis

**Current Pattern**: `.gitignore` contains **NO** entry for BenchmarkDotNet.Artifacts.

**Recommendation (Iter-141 Audit)**:
- Add explicit line: `BenchmarkDotNet.Artifacts/` (literal dot, not regex escaped)
- This pattern does NOT exist in the current `.gitignore`

**Why This Matters**: When the benchmark suite eventually runs and outputs to `BenchmarkDotNet.Artifacts/results/**/*-report-full.json`, those files could leak into git staging. The lack of an explicit pattern means the directory would be untracked but could be accidentally committed if someone does `git add .` in the repo root.

**Action**: Add `BenchmarkDotNet.Artifacts/` to `.gitignore` line 93 (after `docs/sessions/DINOForgeCompanion/`).

---

## (d) Per-Benchmark Category State

### ContentLoaderBenchmarks

**Implemented Benchmarks** (in `src/Tests/Benchmarks/ContentLoaderBenchmarks.cs`):
- `Load_Packs_Manifest_Parsing()` — Synthetic YAML parsing (parameter: PackCount ∈ [1, 5, 10, 25, 50])
- `Load_Packs_Registry_Population()` — Dictionary insertion for registry entries
- `Load_Packs_Full_Pipeline()` — End-to-end manifest + registry + validation

**Baseline Status**: Hardcoded placeholder (50M-150M ns range)  
**Real Measurement**: NOT YET CAPTURED

### RegistryBenchmarks

**Implemented Benchmarks** (in `src/Tests/Benchmarks/RegistryBenchmarks.cs`):
- `QueryUnits()` — Query 1000 units by faction + role
- `RegisterBulkUnits()` — Register 100 units with conflict detection

**Baseline Status**: Hardcoded placeholder (5M-10M ns range)  
**Real Measurement**: NOT YET CAPTURED

### JSON-RPC, HMAC, StringBuilder Categories

**Status**: Not implemented. No benchmarks for:
- JSON-RPC round-trip serialization (Protocol.JsonRpcMessage)
- HMAC/token validation performance
- StringBuilder allocation patterns in CLI/PackCompiler

These categories were noted in CLAUDE.md as future observability goals but are not yet wired.

---

## (e) Critical Issue: Path Mismatch in CI Workflow

**Benchmark Project Location**: `src/Tests/Benchmarks/DINOForge.Benchmarks.csproj`

**CI Workflow Path** (`.github/workflows/benchmarks.yml` line 48):
```bash
dotnet run --project src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj -- --configuration Release
```

**Status**: ❌ **BROKEN** — CI references `src/Tools/Benchmarks/` but the project is at `src/Tests/Benchmarks/`.

**Impact**:
- When `benchmarks.yml` runs on GitHub Actions, it will NOT find the project
- Step "Run benchmarks" silently skips (line 47: `if [ -d "src/Tools/Benchmarks" ]` returns false)
- No BenchmarkDotNet.Artifacts directory is ever created
- Regression gate (step "Check performance regression") exits gracefully with "No benchmark results found"
- Baseline is never updated; `latest.json` is never generated

**Result**: The benchmark infrastructure appears to exist but does not execute.

---

## (f) Runability Assessment

**Local Execution**: The benchmark suite CAN be run locally:

```bash
dotnet run --project src/Tests/Benchmarks/DINOForge.Benchmarks.csproj -c Release
```

**Expected Output**:
- BenchmarkDotNet will create `BenchmarkDotNet.Artifacts/results/` with `.json` report files
- Console output shows benchmark timing statistics
- On first run, a new baseline is established
- Subsequent runs compare against baseline

**CI Execution**: The suite CANNOT be run via `benchmarks.yml` without fixing the path.

**Cleanup Recommendation**:
1. Fix `.github/workflows/benchmarks.yml` line 48: change `src/Tools/Benchmarks/...` to `src/Tests/Benchmarks/...`
2. Add `BenchmarkDotNet.Artifacts/` to `.gitignore`
3. On next manual workflow_dispatch run, baselines will be captured and stored in `docs/benchmarks/latest.json`

---

## Summary Table

| Aspect | Status | Size | Last Updated |
|--------|--------|------|--------------|
| BenchmarkDotNet.Artifacts on disk | Not present | 0 MB | N/A |
| Committed baseline (docs/benchmarks/baseline.json) | Placeholder values | 317 B | 2026-03-29 |
| Committed index (docs/benchmarks/index.md) | Exists | 2.5 KB | 2026-03-29 |
| CI Workflow (benchmarks.yml) | **BROKEN PATH** | 4.5 KB | 2026-04-26 |
| .gitignore BenchmarkDotNet pattern | Missing | N/A | N/A |
| Can run locally? | Yes | — | — |
| Can run on CI? | No (path mismatch) | — | — |

---

## Recommendations (Priority Order)

1. **CRITICAL**: Fix `.github/workflows/benchmarks.yml` line 48:
   ```diff
   -  dotnet run --project src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj -- --configuration Release
   +  dotnet run --project src/Tests/Benchmarks/DINOForge.Benchmarks.csproj -c Release
   ```

2. **HIGH**: Add `.gitignore` pattern (line 93):
   ```
   BenchmarkDotNet.Artifacts/
   ```

3. **MEDIUM**: Update `docs/benchmarks/index.md` line 58 — fix the local run command path to match the corrected location.

4. **MEDIUM**: Capture real baseline on next CI run (manual trigger via GitHub UI after fix #1):
   - Workflow will populate `docs/benchmarks/baseline.json` with real measurements
   - `docs/benchmarks/latest.json` will be updated automatically
   - Index.md will then display actual benchmark history

5. **LOW**: Extend benchmarks to JSON-RPC and StringBuilder categories if perf regression tracking is expanded.

---

## No Git Staging Leaks

✅ **Finding**: No BenchmarkDotNet artifacts are currently staged in git.  
✅ **Reason**: The artifacts directory has never been generated (due to path mismatch in CI, no local runs recorded).  
✅ **Safety**: `.gitignore` patterns cover `bin/`, `obj/` which isolate compiled assemblies.

**Action**: Add explicit `BenchmarkDotNet.Artifacts/` pattern to prevent future accidental leaks.
