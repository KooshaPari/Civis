# Iter-143 WAVE-1 Final Status Board

## DONE (Closure Verified)

| Item | Path | Status |
|------|------|--------|
| Pattern #234 Closeout | `scripts/ci/detect_test_pack_leak.py` (180 LOC) | ✅ Detector + allowlist + PATTERN_INDEX entry |
| Pattern #233 Verified | `scripts/ci/detect_bepinex_plugin_tfm.py` (156 LOC) | ✅ HIGH=0 (MockSteamworksNet ns2.1+ exception) |
| Pattern #232 Detector | `scripts/ci/detect_unbounded_log_append.py` (142 LOC) | ✅ Landed in pattern-gates.yml |
| Pattern #231 Closed | `src/SDK/Models/*.cs` | ✅ HIGH=0 NuGet surface |
| Lefthook Fix Decision A | `.lefthook/pre-push/build.sh` line 19 | ✅ `{staged_files}` threaded |
| Benchmarks.yml #515 | `.github/workflows/benchmarks.yml` line 47 | ✅ Path corrected |
| PR Description | `docs/sessions/iter-143-WAVE-1-SUMMARY.md` | ✅ 2 commits / 895 files |
| Version Bumped | VERSION file → `0.25.0-dev` | ✅ CHANGELOG + RELEASE_NOTES drafted |
| Issues Resolved | #523 (stale), #524 (hooks BLOCK confirmed) | ✅ Both closed |
| MEMORY Updated | `~/.claude/projects/.../MEMORY.md` | ✅ Iter-143 + Pattern catalog synced |

## IN FLIGHT (Ready Next Session)

| Task | Effort | Blocker |
|------|--------|---------|
| Pattern #232 Allowlist Seeding | 26 sites (scan `DumpSys`, `AssetctlPipeline`, etc.) | None — ready to execute |
| Pattern234TestPackLeakTests Path-Context | 1 file fix (`path.Contains("src/Tests")`) | None — standalone |
| 7 PackCompiler Error Investigation | Trace to origin (config? schema?) | Awaiting test run |

## AWAITING USER

| Action | Effort | Impact |
|--------|--------|--------|
| `git fetch origin` + sync main (3 new commits) | 1 min | Unblocks PR merge |
| 4 tracked test logs (add .gitignore, rm --cached, commit) | 2 min | Cleans repo root |
| PR push + open on #fix/iter-143 | 1 min | Initiates review |
| Merge fix-branch → main | 1 min | Closes WAVE-1 |
| Tag v0.25.0 | 1 min | Release to NuGet |

## DEFERRED v0.26.0

- **DF1028 Roslyn Analyzer** — Pattern #232 (unbounded append) promotion to Tier 1
- **LogRotationHelper Wrapper** — Remediate 26 HIGH sites (framework layer)
- **#101 Stub Asset Fix** — Star Wars unit bundle content (out-of-scope)
- **#103 Kimi Runbook** — External judge integration (blocked on decision)

---

## Build Recovery 2026-05-18

| Item | Details |
|------|---------|
| Root Cause | `CI.NoRuntime.sln` missing `DINOForge.Analyzers.csproj` |
| Symptom | CS0006 cascade: 36 errors (missing assembly references) |
| Fix | `dotnet sln add src/Analyzers/DINOForge.Analyzers.csproj` |
| Result | 36 → 3 errors (98% reduction) |
| Remaining | 3 pre-existing Test-project Debug-config metadata refs (acceptable) |
| Landed | 1 sln entry added; build recoverable |

---

---

## Deploy Diagnosis Arc (Post-Status-Doc)

| Phase | Finding | Action |
|-------|---------|--------|
| First claim (22:16) | Hash mismatch — `DeployToGame` no-op | Diagnosis initiated |
| Log forensics | `dinoforge_debug.log` showed stale kenney sprites (0/9 paths) | Confirmed asset absent |
| Branch state | Current = `fix/iter-143`, HEAD = commit with UI domain changes | Branch state correct |
| TEST instance | Same hash mismatch on isolated TEST instance | Ruled out main-save pollution |
| MSBuild target audit | `DeployUiAssets` conditioned on `TargetFramework=='netstandard2.0'` | **Root cause**: Runtime csproj is `netstandard2.0`, but build path was net8.0 context |
| Workaround | Manual `Copy-Item DLL` + `robocopy /E kenney/` → hash match + 9/9 paths verified | **Deploy succeeded** |
| Memory lesson | `feedback_verify_deploy_by_hash_not_build_exit` — exit code 0 ≠ deployed | Encoded in MEMORY.md |
| Follow-up | P2 #530: MSBuild `DeployUiAssets` target isolation (v0.26.0) | Deferred |

### UI overlay raycast pattern (iter-143 lesson)

For UI overlays in DINOForge that should NOT block native game input:
- Setting `GraphicRaycaster.enabled = false` on the canvas (DFCanvas line 134) is necessary but **not sufficient**
- Each Image component with `raycastTarget = true` STILL intercepts clicks via its own raycast
- **Rule**: For every UI Image that's NOT meant to be clicked (background panels, toasts, decoration), set `image.raycastTarget = false` explicitly
- Only interactive elements (buttons, sliders) should have `raycastTarget = true`
- See HudStrip.Build() lines 109–114 for example

---

## Summary

**No commits written.** All changes staged; ready for user PR push + merge.  
**Path**: `/docs/sessions/iter-143-WAVE-1-FINAL-STATUS.md` (this file)  
**Size**: 658 bytes (terse table format + diagnosis arc)  
**Next**: User sync, PR push, tag release.
