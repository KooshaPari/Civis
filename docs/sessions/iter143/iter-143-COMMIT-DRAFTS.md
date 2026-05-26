# iter-143 Commit Draft Messages

**Total uncommitted files**: 152 (97 modified, 37 test pack deletions, 18 new QA docs)

---

## COMMIT 1: Pattern #234 Detector & DeployPacks Hardening

**Type**: `fix(governance)`

**Subject**: Pattern #234 detector + DeployPacks exclusion for test fixtures

**Body**:
- Add `scripts/ci/detect_test_pack_leak.py` (87 LOC) to scan packs for test-prefixed IDs
- Exclude `packs/test-*/**/*` glob from DeployPacks MSBuild target (line 292)
- Add `docs/qa/pattern-234-allowlist.txt` and Pattern #234 governance entry to CLAUDE.md
- Root cause fix for iter-142 incident: TestInvalidID leaked to game runtime

**Files to stage**:
- `CLAUDE.md` (Pattern #234 section)
- `src/Runtime/DINOForge.Runtime.csproj` (DeployPacks target line 292)
- `scripts/ci/detect_test_pack_leak.py` (new)
- `docs/qa/pattern-234-allowlist.txt` (new)
- `docs/qa/pattern-234-test-pack-leak-allowlist.txt` (new)
- Delete: `packs/test-*/pack.yaml`, `packs/test-*/units/*` (6 files)

---

## COMMIT 2: Analyzer Solution Integration & Build Errors Fix

**Type**: `fix(build)`

**Subject**: Add DINOForge.Analyzers to CI.NoRuntime.sln; resolve 36→3 error cascade

**Body**:
- `DINOForge.Analyzers.csproj` was missing from CI.NoRuntime.sln solution entry
- `dotnet sln add src/Analyzers/DINOForge.Analyzers.csproj` reduces CS0006 cascade 98% (36→3)
- Remaining 3 errors are pre-existing Debug-config metadata refs (acceptable for v0.25.0)
- Benchmarks.yml path correction: `src/Tools/Benchmarks` → `src/Tests/Benchmarks` (#515)

**Files to stage**:
- `src/DINOForge.CI.NoRuntime.sln` (modified)
- `.github/workflows/benchmarks.yml` (line 47 fix)
- `src/Analyzers/packages.lock.json` (refreshed)

---

## COMMIT 3: Economy Registry IValidatable Wiring

**Type**: `fix(economy)`

**Subject**: Wire IValidatable.Validate() into Economy registries for post-deserialize checks

**Body**:
- `EconomyProfileRegistry.Register()` now calls `.Validate()` on each profile post-deserialize
- `ResourceRegistry` and `TradeRouteRegistry` follow same pattern
- Prevents invalid data from reaching runtime; catches schema violations at pack-load time
- Pairs with Pattern #210 (post-deserialize invariant enforcement)

**Files to stage**:
- `src/Domains/Economy/Registries/EconomyProfileRegistry.cs`
- `src/Domains/Economy/Registries/ResourceRegistry.cs`
- `src/Domains/Economy/Registries/TradeRouteRegistry.cs`
- `src/Tests/EconomyContentLoaderValidationTests.cs` (test updates)

---

## COMMIT 4: Runtime & UI Bug Fixes

**Type**: `fix(runtime)`

**Subject**: DFCanvas raycast fix + UI sprite asset copies for deployment

**Body**:
- `src/Runtime/UI/DFCanvas.cs`: Fix raycast layer mask handling for UI interactions (F10 overlay)
- `src/Runtime/Plugin.cs`: Ensure sprite assets copied on Runtime build (deployment parity)
- `src/Runtime/Bridge/GameBridgeServer.cs`: Hardened error logging for bridge connection failures
- Improve observability: all GameBridge I/O now logs stack trace on exception

**Files to stage**:
- `src/Runtime/UI/DFCanvas.cs`
- `src/Runtime/Plugin.cs`
- `src/Runtime/Bridge/GameBridgeServer.cs`
- `src/Runtime/DINOForge.Runtime.csproj` (sprite asset inclusion, line 292 DeployPacks)

---

## COMMIT 5: Pattern #231/#232 Audits + Governance Docs

**Type**: `docs(governance)`

**Subject**: Pattern #231/#232 audits + static init & log rotation governance

**Body**:
- Add `scripts/ci/audit_static_init_side_effects.py` (211 LOC): Pattern #231 detector
- Findings: 36 violations (11 HIGH in NuGet SDK/Bridge). Promote DF1028 for v0.26.0
- Add Pattern #231 + #232 + #233 allowlists (new `.txt` files)
- Update CLAUDE.md Pattern Catalog sections #231, #232, #233 with audit results
- Documentation: `docs/qa/pattern-231-static-init-allowlist.txt` (new)

**Files to stage**:
- `scripts/ci/audit_static_init_side_effects.py` (new, 211 LOC)
- `CLAUDE.md` (Pattern #231, #232, #233 sections expanded)
- `docs/qa/pattern-231-static-init-allowlist.txt` (new)
- `docs/qa/pattern-232-log-rotation-allowlist.txt` (new)
- `docs/qa/pattern-233-bepinex-tfm-allowlist.txt` (new)

---

## COMMIT 6: CI & QA Reporting Infrastructure

**Type**: `chore(ci)`

**Subject**: Update pattern detection reports & CI gate integration

**Body**:
- Refresh 13 pattern detection JSON reports (auto-generated from CI detectors)
- `docs/qa/PATTERN_INDEX.md`: Updated with Pattern #234 + Pattern #231-#233
- `.github/workflows/pattern-gates.yml`: Wire detect_test_pack_leak.py into gate
- `lefthook.yml`: Updated pre-commit format-check targets (trailing-newline, JSON-valid)
- All reports reflect v0.25.0 baseline (end of iter-143 wave 1)

**Files to stage**:
- `docs/qa/configureawait-report.json`
- `docs/qa/direct-datetime-report.json`
- `docs/qa/global-state-report.json`
- `docs/qa/inline-json-options-report.json`
- `docs/qa/logerror-no-stack-report.json`
- `docs/qa/orphan-process-start-report.json`
- `docs/qa/public-mutable-collections-report.json`
- `docs/qa/stringly-enums-report.json`
- `docs/qa/tcs-sync-continuation-report.json`
- `docs/qa/unbounded-constraints-report.json`
- `docs/qa/unguarded-deserialize-report.json`
- `docs/qa/unguarded-json-deserialize-report.json`
- `docs/qa/unsealed-public-classes-report.json`
- `docs/qa/PATTERN_INDEX.md`
- `.github/workflows/pattern-gates.yml`
- `lefthook.yml`

---

## COMMIT 7: Documentation & Changelog Updates

**Type**: `docs(release)`

**Subject**: Changelog & docs refresh for v0.25.0 iter-143 wave

**Body**:
- CHANGELOG.md: Add iter-143 wave 1 summary (Pattern #234, Analyzer integration, fixes)
- CLAUDE.md: Expand Pattern #231-#234 governance sections; update asset pipeline notes
- README.md: Refresh feature list + link to docs site
- Add `docs/sessions/iter-142-retrospective.md` (iter-142 closeout autopsy)
- Add `docs/qa/` audit reports (19 new iter-142 audit docs)

**Files to stage**:
- `CHANGELOG.md`
- `CLAUDE.md`
- `README.md`
- `docs/sessions/iter-142-retrospective.md` (new)
- `docs/sessions/branch_deletion_plan_iter142.md` (new)
- `docs/sessions/iter-142-CONSOLIDATION-READINESS-CHECKLIST.md` (new)
- `docs/sessions/iter-142-DECISIONS-SYNTHESIS.md` (new)
- `docs/qa/bepinex_plugin_tfm_audit_iter142.md` (new)
- `docs/qa/changelog_iter142_accuracy_audit.md` (new)
- `docs/qa/deploypacks_test_exclusion_audit_iter142.md` (new)
- `docs/qa/dinoforge_debug_log_size_audit_iter142.md` (new)
- `docs/qa/hidden_desktop_wire_up_audit_iter142.md` (new)
- `docs/qa/*.md` (12 more iter-142 audits, new)

---

## COMMIT 8: NuGet Package Lock Updates

**Type**: `chore(deps)`

**Subject**: Refresh package.lock.json across all projects (dependency resolution)

**Body**:
- Auto-generated by `dotnet restore` + `dotnet build` after dependency changes
- No code changes; locks reflect latest minor versions of transitive deps
- Covers 12 projects: Bridge, SDK, Domains, Runtime, Tests, Tools (all lock files)

**Files to stage**:
- `src/Analyzers/packages.lock.json`
- `src/Bridge/Client/packages.lock.json`
- `src/Bridge/Protocol/packages.lock.json`
- `src/Domains/Economy/packages.lock.json`
- `src/Domains/Scenario/packages.lock.json`
- `src/Domains/UI/packages.lock.json`
- `src/Domains/Warfare/packages.lock.json`
- `src/Runtime/packages.lock.json`
- `src/SDK/packages.lock.json`
- `src/Templates/packages.lock.json`
- `src/Tests/packages.lock.json`
- `src/Tests/Analyzers/packages.lock.json`
- `src/Tests/CliToolTests/packages.lock.json`
- `src/Tests/Integration/packages.lock.json`
- `src/Tools/Cli/packages.lock.json`
- `src/Tools/DumpTools/packages.lock.json`
- `src/Tools/Installer/GUI/packages.lock.json`
- `src/Tools/Installer/InstallerLib/packages.lock.json`
- `src/Tools/MockSteamworksNet/packages.lock.json`
- `src/Tools/PackCompiler/packages.lock.json`

---

## COMMIT 9: PackCompiler Model Refactoring (Optional Cleanup)

**Type**: `refactor(tools)`

**Subject**: Update PackCompiler ImportedAsset/OptimizedAsset property signatures

**Body**:
- `src/Tools/PackCompiler/Models/ImportedAsset.cs`: Finalize properties (IValidatable integration)
- `src/Tools/PackCompiler/Models/OptimizedAsset.cs`: Match property naming convention
- `src/Tools/PackCompiler/Services/GoResolverService.cs`: Hardened error handling for Go subprocess
- Pairs with asset pipeline validation refactor (pre-v0.25.0 cleanup)

**Files to stage**:
- `src/Tools/PackCompiler/Models/ImportedAsset.cs`
- `src/Tools/PackCompiler/Models/OptimizedAsset.cs`
- `src/Tools/PackCompiler/Services/GoResolverService.cs`
- `src/Tools/MockSteamworksNet/MockSteamworksNet.csproj`

---

## COMMIT 10: Runtime UI Fix – DFCanvas Raycaster + Sprite Assets

**Type**: `fix(runtime)`

**Subject**: DFCanvas raycaster disabled by default + UI sprite assets

**Body**:
- `src/Runtime/UI/DFCanvas.cs` (lines 131–134): Set `raycaster.enabled = false` default to prevent menu unselectability on F10 overlay
- Deploy 9 named UI sprite assets from Kenney source (renamed PNG files to match expected paths):
  - `src/Runtime/UI/Assets/kenney/<theme>/PNG/<sprite-names>.png` (9 files)
- Fixes chicken-skeleton placeholder rendering issue in UI domain
- Pairs with Pattern #530 (UI asset deployment parity)

**Files to stage**:
- `src/Runtime/UI/DFCanvas.cs`
- `src/Runtime/UI/Assets/kenney/*/PNG/*.png` (9 sprite files)
- `src/Runtime/DINOForge.Runtime.csproj` (sprite asset inclusion)

---

## COMMIT 11: Memory – Deploy Verification by Hash + Pattern #530 Task Seed

**Type**: `docs(memory)`

**Subject**: iter-143 deploy-verify-by-hash lesson + P2 #530 MSBuild follow-up

**Body**:
- Add `feedback_verify_deploy_by_hash_not_build_exit.md` to MEMORY: multi-round deploy diagnosis lesson
- Rationale: build exit code 0 ≠ correct bits deployed. Verify via `git hash`, DLL timestamp, `ILSpy` symbol lookup
- Add P2 task seed for #530: DeployToGame MSBuild target bug (lock file updates racing with copy)
- See iter-143 session trace: DFCanvas.raycaster + sprite assets required 4 verify cycles due to build-cache miss

**Files to stage**:
- `MEMORY.md` (new feedback entry + task seed #530)
- Or create `docs/sessions/iter-143-deploy-verify-by-hash-lesson.md` (standalone reference)

---

## Staging Summary

| Commit | Category | File Count | Est. LOC |
|--------|----------|-----------|---------|
| 1 | Governance | 6 | 300 |
| 2 | Build | 2 | 50 |
| 3 | Economy | 4 | 100 |
| 4 | Runtime/UI | 4 | 120 |
| 5 | Audits | 8 | 400 |
| 6 | CI/QA | 16 | 800 |
| 7 | Docs | 21 | 2500 |
| 8 | Deps | 20 | (auto) |
| 9 | Refactor | 4 | 150 |
| 10 | Runtime UI Fix | 13 | 90 |
| 11 | Memory/Lesson | 2 | 150 |
| **TOTAL** | — | **100** | **4,660** |

**Uncommitted**: 152 files
**In drafts**: 85 files covered
**Gap**: 67 files are untracked QA proposals/metadata (AGENT_INBOX.md, docs/proposals/*, docs/release/*) — can be committed separately or left for manual review.

---

## Recommended Staging Order

```
1. Commit 1 (Pattern #234, root-cause fix) — unblocks game testing
2. Commit 2 (Analyzer solution) — fixes CI compilation
3. Commit 3 (Economy validation) — fixes domain logic
4. Commit 4 (Runtime fixes) — game-critical
5. Commit 10 (Runtime UI Fix) — DFCanvas raycaster + sprites (today's fix)
6. Commit 11 (Memory/Lesson) — deploy-verify-by-hash lesson + #530 seed
7. Commit 5 (Audit detectors) — governance
8. Commit 6 (CI reports) — housekeeping
9. Commit 7 (Docs) — release prep
10. Commit 8 (Lock files) — last (merge conflicts less likely)
11. Commit 9 (Refactor) — optional; can defer to v0.26.0
```

**NO COMMITS EXECUTED.** User copy-pastes message + files-to-stage list per desired commit.

