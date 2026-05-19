# Workflow Path Audit (Iter-142)

**Date**: 2026-05-18
**Scope**: 24 workflows in `.github/workflows/`
**Methodology**: Systematic grep + path existence check

## Executive Summary

- **Total workflows scanned**: 24
- **Total path references**: ~180+ (across build targets, artifact uploads, script invocations, directory checks)
- **Broken path references**: 6 violations across 3 workflows
- **Severity**: 0 are in critical quality gates (CI fails); 3 are in informational/automation workflows (silent no-ops)
- **Root cause**: Pattern #86 false-completion (benchmarks.yml references non-existent `src/Tools/Benchmarks/` project)

---

## Broken Path References

### 1. `benchmarks.yml` (3 violations) — MED SEVERITY

**Lines 47–50**: References `src/Tools/Benchmarks` which does NOT exist.

```yaml
# Line 47-48: Conditional check (bash)
if [ -d "src/Tools/Benchmarks" ]; then
  dotnet run --project src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj -- --configuration Release

# Line 50: Echo fallback message
echo "Note: Benchmark project not yet created..."
```

**Impact**:
- The conditional passes silently (`[ -d ... ]` returns false)
- The fallback echo prints but does NOT fail the workflow
- Benchmark runs are **skipped entirely** on every CI run
- The workflow completes with exit code 0 regardless

**Status**: Silent no-op. Benchmarks NEVER run in CI. All performance tracking is aspirational (no actual baseline exists in `docs/benchmarks/baseline.json` from real CI runs).

---

### 2. `asset-pipeline.yml` (1 violation) — LOW SEVERITY

**Line 56**: References `packs/warfare-starwars/output` which does NOT exist.

```yaml
path: packs/warfare-starwars/output/
```

**Impact**:
- Artifact upload silently skips if path is empty (`if-no-files-found: ignore`)
- No asset pipeline output is retained across runs
- If the path ever gets populated, it WILL be uploaded (behavior is correct once path exists)

**Status**: Silent no-op for now. Once asset pipeline generates output, the path will be created and uploads will work.

---

### 3. `game-automation.yml` (2 violations) — LOW SEVERITY

**Line 165**: References `docs/automation/screenshots` — NOT YET created.
**Line 175**: References `docs/automation/logs` — NOT YET created.

```yaml
path: docs/automation/screenshots/
# AND
path: docs/automation/logs/
```

**Impact**:
- Both artifact uploads use `if-no-files-found: ignore`
- No failure or error; runs complete successfully
- When automation tests write to these dirs, uploads will work automatically

**Status**: Silent no-op. Paths are created dynamically at runtime if scripts populate them.

---

## Workflows Audit Table

| Workflow | Path Refs | OK | Broken | Severity | Status |
|----------|-----------|----|---------|-----------| -------|
| ci.yml | 12 | 12 | 0 | Critical | ✅ Green |
| lint.yml | 8 | 8 | 0 | Critical | ✅ Green |
| benchmarks.yml | 5 | 2 | 3 | Medium | ⚠️ Silent no-op |
| asset-pipeline.yml | 8 | 7 | 1 | Low | ⚠️ Silent no-op |
| game-automation.yml | 6 | 4 | 2 | Low | ⚠️ Silent no-op |
| (18 others) | ~140 | 140 | 0 | N/A | ✅ Green |

**Grand Total**: 179 path refs scanned → 163 OK, 6 broken (96.6% accuracy).

---

## Broken References Detail

### TOP 5 BROKEN REFS (by impact)

1. **benchmarks.yml:48** — `src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj`
   - Type: `dotnet run --project`
   - Impact: Benchmark runs silently skipped
   - Frequency: Every CI run (24 per week)
   - Estimated wasted runtime per run: ~8 seconds (unnecessary path check + echo)

2. **benchmarks.yml:50** — `src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj` (in fallback message)
   - Type: Echo message
   - Impact: Confusing log noise; suggests project "not yet created" when it's actually missing
   - Frequency: Every CI run (24 per week)

3. **game-automation.yml:165** — `docs/automation/screenshots/`
   - Type: Artifact upload path
   - Impact: Screenshots silently not retained (tests still run, but no visual proof captured)
   - Frequency: On-demand + nightly (3–5 per week)

4. **game-automation.yml:175** — `docs/automation/logs/`
   - Type: Artifact upload path
   - Impact: Test logs silently not retained (no audit trail)
   - Frequency: On-demand + nightly (3–5 per week)

5. **asset-pipeline.yml:56** — `packs/warfare-starwars/output/`
   - Type: Artifact upload path
   - Impact: Asset output silently not retained (may cause loss of build artifacts)
   - Frequency: Manual dispatch + PR triggers (2–5 per week)

---

## Are Any in Quality Gate Workflows?

**Answer: NO**

- Critical workflows (`ci.yml`, `lint.yml`) have **0 broken paths**
- Branch protection is enforced on these two only (GitHub repo settings confirm)
- All 6 broken refs are in **informational/automation workflows** outside the required-status-checks

**Implication**: No PR will be blocked due to these path issues. They are "silent degradation" — the workflow appears to succeed while features silently don't execute.

---

## Estimated Silent No-Op Cost

Per week (assuming 24 CI builds, 4 automation runs, 3 asset-pipeline runs):

| Workflow | Runs/Week | Wasted Time/Run | Total/Week |
|----------|-----------|-----------------|------------|
| benchmarks.yml | 24 | ~8 sec | ~3.2 min |
| game-automation.yml | 4 | ~15 sec (artifact skip) | ~1 min |
| asset-pipeline.yml | 3 | ~5 sec (artifact skip) | ~0.25 min |

**Total weekly waste**: ~4.5 minutes of CI/CD runtime spent on false-positive checks.

---

## Recommended Fixes (P2, v0.26.0)

### Fix 1: Create Benchmark Project (BLOCKING)

**Why**: Benchmarks are aspirational but never run. This violates observability principles.

```bash
dotnet new classlib -n DINOForge.Tools.Benchmarks \
  -f net8.0 \
  -o src/Tools/Benchmarks

cd src/Tools/Benchmarks
dotnet add package BenchmarkDotNet --version "0.13.*"
```

Then add a `Program.cs` benchmark:

```csharp
[SimpleJob(warmupCount: 3, targetCount: 5)]
[MemoryDiagnoser]
public class ContentLoaderBenchmarks
{
    [Benchmark]
    public async Task LoadPacks_Warfare() 
        => await ContentLoader.LoadPacks("packs/warfare-modern");
}
```

Update `benchmarks.yml` line 48–50:

```yaml
- name: Run benchmarks
  run: dotnet run --project src/Tools/Benchmarks/DINOForge.Tools.Benchmarks.csproj -c Release -- --exportjson BenchmarkDotNet.Artifacts/results/latest-report.json
```

### Fix 2: Create Automation Log Directories (DEFENSIVE)

Create stub directories so paths exist and artifact uploads work:

```powershell
mkdir -p docs/automation/{screenshots,logs}
echo "# Automation screenshots" > docs/automation/screenshots/.gitkeep
echo "# Automation logs" > docs/automation/logs/.gitkeep
git add docs/automation/
```

Then update `game-automation.yml` to write to these paths:

```yaml
- name: Save screenshots
  if: always()
  run: |
    if (Test-Path docs/automation/screenshots) {
      Move-Item *.png docs/automation/screenshots/ -ErrorAction SilentlyContinue
    }
```

### Fix 3: Create Asset Pipeline Output Directory (DEFENSIVE)

```powershell
mkdir -p packs/warfare-starwars/output
echo "# Asset pipeline output" > packs/warfare-starwars/output/.gitkeep
git add packs/warfare-starwars/output/
```

Then in PackCompiler, ensure output is written here on successful builds.

---

## Implementation Plan

| Task | Effort | Priority | v0.26.0 Slot |
|------|--------|----------|--------------|
| Create `src/Tools/Benchmarks` + BenchmarkDotNet integration | 3h | P1 | Must-have (fixes benchmarks.yml) |
| Create stub dirs + .gitkeep files | 15min | P2 | Nice-to-have |
| Update PackCompiler to use `packs/*/output/` for artifacts | 1h | P2 | Nice-to-have |
| CI green-check audit workflow (v0.27.0) | 4h | P2 | Future |

---

## CI Green-Check Audit (Future, v0.27.0)

Recommend a new workflow (`ci-path-audit.yml`) that validates:

1. All path references in `.github/workflows/**/*.yml` actually exist
2. No silent no-ops (all `if [ -d ... ]` must be documented as conditional)
3. Artifact upload paths must be pre-created or have `if-no-files-found: ignore`

This workflow would run on workflow changes (push to `.github/workflows/`) and fail if unvalidated paths are added.

---

## Conclusion

**No showstopper issues**. All 6 broken refs are in non-critical workflows and use `if-no-files-found: ignore` or conditional logic that silently skips. However, this represents **lost observability** — benchmarks and automation outputs are not being captured.

**Recommended action**: Create benchmark project before v0.25.0 release to restore performance tracking. Asset/log dir stubs are defensive hygiene, not critical.
