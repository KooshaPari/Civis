# Phase 2 Merge Conflict Prediction (iter142)

**Date**: 2026-05-18  
**Branch Delta**: `fix/handle-connect-iter142` (2 commits ahead) vs `origin/main` (51 commits behind)  
**Purpose**: Predict merge conflicts before `git merge origin/main`

## Summary

| Metric | Value |
|--------|-------|
| Commits on origin/main | 51 (confirmed) |
| Files touched on origin/main | 7,108 |
| Files touched on ced0dccf commit | 878 |
| **Intersection (likely conflicts)** | **282 files** |
| Estimated Effort | **HIGH** |

---

## Finding (a): Commit Count Verification

✓ **CONFIRMED**: 51-commit delta accurate.  
Output: `2 51` (2 ahead, 51 behind).

---

## Finding (b): Files Touched on origin/main

**Total**: 7,108 files across 51 commits.

Scope includes:
- 15+ new `.github/workflows/` pattern gate files (Roslyn analyzers, benchmarks, schema drift, etc.)
- 50+ new `scripts/ci/` and `docs/qa/` quality audit scripts
- 100+ new Roslyn analyzer definitions and tests (`src/Analyzers/`)
- 200+ test files (parameterized tests, game launch tests, mocks, integration tests)
- 50+ docs/sessions/ and docs/proof/ quality artifacts
- packs/ and schemas/ expanded

**Key insight**: origin/main has been heavily extended with CI/QA/testing infra (v0.24.0 release prep).

---

## Finding (c): Files Touched on ced0dccf (Single Commit)

**Total**: 878 files on the `fix/handle-connect-iter142` branch tip.

Scope includes:
- ✓ Core fix: `src/Bridge/Client/GameClient.cs`, `GameClientOptions.cs`, `GameProcessManager.cs` (HandleConnect implementation)
- ✓ Bridge protocol updates: `src/Bridge/Protocol/*.cs` (VerifyResult, GameStatus, JsonRpcMessage, etc.)
- ✓ SDK + Domain plugins: Economy, Scenario, UI (all .cs + .csproj + packages.lock.json)
- ✓ Runtime updates: Bridge/, UI/, Aviation/, Asset services
- ✓ Tests: GameLaunch tests, Integration tests, ParameterizedTests, Benchmarks
- ✓ MCP server: `src/Tools/DinoforgeMcp/` (Python proof system, policy, signing, aggregator)
- ✓ Docs: user-journeys, proof examples, vitepress config, docs index
- ✓ Top-level: CHANGELOG.md, CLAUDE.md, README.md, VERSION, Directory.Build.props

**Key insight**: ced0dccf is a comprehensive MEGA-COMMIT consolidating 13+ iterations of bridge handshake work, proof system wiring, domain plugin expansion, and test infrastructure sprawl into one logical changeset.

---

## Finding (d): Intersection (Likely Conflict Files)

**Total**: 282 files in common (both touched on ced0dccf AND on origin/main in 51 commits).

These are files that:
1. Were modified/created on ced0dccf (`fix/handle-connect-iter142` tip)
2. Were also modified/created elsewhere in the 51 commits since the branch point

**Conflict surface confirmed**: 282 files have diverged history.

---

## Finding (e): Top 10 Likely Conflict Files (Classified)

| # | File | Classification | Conflict Type | Severity |
|---|------|-----------------|---------------|----------|
| 1 | `CHANGELOG.md` | Doc (text) | Both sides appended entries for v0.24.0 release | MED |
| 2 | `CLAUDE.md` | Governance (text) | Both sides extended governance rules, pattern catalog, asset pipeline | MED |
| 3 | `src/Bridge/Client/GameClient.cs` | Source (.cs) | ced0dccf: HandleConnect handshake; origin/main: likely async/CT fixes | HIGH |
| 4 | `src/Bridge/Protocol/JsonRpcMessage.cs` | Source (.cs) | Protocol evolution, both sides may have schema changes | HIGH |
| 5 | `src/Bridge/Client/packages.lock.json` | Lockfile (text) | NuGet dependency pin drift (v0.24.0 tag on ced0dccf) | MED |
| 6 | `src/Tests/Integration/TestResults/.../*.xml` | Test artifacts | Both sides generate new test runs, directory tree diverged | LOW (auto-generated) |
| 7 | `docs/.vitepress/config.mts` | Config (text) | Both sides added new doc sections (proof, QA, patterns) | MED |
| 8 | `src/Runtime/Bridge/GameBridgeServer.cs` | Source (.cs) | Critical async path; both sides likely touched for async hygiene | HIGH |
| 9 | `VERSION` | Version (text) | ced0dccf: pinned to v0.24.0; origin/main: likely incremented | HIGH |
| 10 | `Directory.Build.props` | MSBuild (text) | .NET 11 TFM pins, game path configuration | MED |

---

## Finding (f): Estimated Merge Effort

**Classification**: **HIGH**

**Rationale**:

1. **282 conflicting files** is substantial (26% of ced0dccf's changeset).
2. **Critical hotspots** (GameClient.cs, JsonRpcMessage.cs, GameBridgeServer.cs) are core bridge protocol—require careful manual resolution.
3. **Version conflict** (VERSION file) signals these branches represent different release states (ced0dccf is v0.24.0 tag-level work; origin/main likely already tagged/incremented).
4. **Governance conflict** (CLAUDE.md) has accumulated pattern catalog entries on both sides; will require intelligent merging of allowlists and pattern definitions.
5. **Test artifact divergence** (282 files includes many auto-generated test runs, BenchmarkDotNet artifacts) — these are safe to delete post-merge, but they inflate the conflict count.
6. **No automation guardrails**: Conflict resolution will require manual inspection and semantic understanding (not just text merging).

---

## Finding (g): Recommended Merge Strategy

**Recommendation**: **Three-phase explicit merge + rebase**

### Phase 1: Prepare Baseline
```powershell
# On fix/handle-connect-iter142, create a merge commit marker:
git merge --no-ff origin/main --no-commit --no-ff
# DO NOT auto-complete; stop and inspect
```

### Phase 2: Resolve High-Risk Conflicts
Manually resolve (in order of risk):
1. **VERSION** → Keep v0.24.0 from ced0dccf (it's the intended release)
2. **src/Bridge/Client/GameClient.cs** → 3-way diff + semantic review (HandleConnect + async fixes)
3. **src/Bridge/Protocol/JsonRpcMessage.cs** → Protocol evolution; ensure ced0dccf's wire format is preserved
4. **GameBridgeServer.cs** → Async safety; apply both sets of changes without duplication
5. **CHANGELOG.md** → Append origin/main entries below ced0dccf's v0.24.0 section
6. **CLAUDE.md** → Merge pattern catalog entries (union of allowlists)

### Phase 3: Accept Auto-Resolved Files
- Lockfiles (packages.lock.json) → Accept ced0dccf version (it's more recent)
- Docs (docs/.vitepress/config.mts) → Merge doc section additions
- Test artifacts (*.xml, *.log, TestResults/) → Delete all (auto-generated; not worth conflict resolution)

### Alternative: Cherry-Pick Mode (Lower Risk)
If manual resolution becomes intractable:
```powershell
# Reset to fix/handle-connect-iter142
git merge --abort

# Cherry-pick only the highest-value commits from origin/main:
# - Governance updates (CLAUDE.md additions, pattern catalog)
# - Critical bug fixes (any async/CT threading fixes)
# - Exclude: test artifacts, docs/proof/, QA audits (can be rebased later)

git cherry-pick <selective-commit-shas>
```

---

## Conflict Resolution Checklist

- [ ] Verify VERSION file: keep `v0.24.0` (ced0dccf intent)
- [ ] Three-way diff GameClient.cs + GameBridgeServer.cs (semantic review, not just text)
- [ ] Protocol schema consistency (JsonRpcMessage, VerifyResult, etc.)
- [ ] Merge allowlists from CLAUDE.md (union of pattern exclusions)
- [ ] Merge CHANGELOG.md entries (append origin/main below ced0dccf's section)
- [ ] Delete all test result artifacts post-merge (safe to regenerate)
- [ ] Run `dotnet build -c Release` to verify no code conflicts remain
- [ ] Run `dotnet test src/DINOForge.sln` (confirm test isolation, no new failures)

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| GameClient.cs semantic conflict (async code + handshake) | HIGH | CRITICAL | Manual 3-way review + test run |
| VERSION file skew | HIGH | MEDIUM | Keep v0.24.0 from ced0dccf |
| Test artifact sprawl (false conflicts) | HIGH | LOW | Delete *.xml, *.log, TestResults/ post-merge |
| Lockfile resolution mismatch | MED | MED | Accept ced0dccf versions (more recent) |
| Pattern catalog duplication (CLAUDE.md) | MED | LOW | Union of allowlists; deduplicate |

---

## Rollback Plan

If merge resolution fails midway:
```powershell
git merge --abort
git reset --hard HEAD  # Return to ced0dccf state
# Notify user; recommend cherry-pick mode or sequential PRs instead
```

---

## Next Steps

1. **User Approval**: Confirm merge strategy (explicit 3-phase vs cherry-pick vs sequential PRs)
2. **Phase 2 Execution**: If approved, execute Phase 1 (stage conflict) + Phase 2 (resolve high-risk) + Phase 3 (accept/delete)
3. **Verification**: Run full build + test suite post-merge to confirm no integration failures
4. **Release**: Tag `v0.24.0` on the merged result (if not already tagged)
