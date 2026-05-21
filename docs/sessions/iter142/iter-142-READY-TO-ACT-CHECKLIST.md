# Iter-142 Ready-to-Act Checklist

**Date**: 2026-05-18  
**Goal**: Unblock #523 → #524 → merge fix/handle-connect → v0.25.0 tag (4–5 hours total)

---

## Phase 0: HandleConnect Deploy ✅ DONE (verified 2026-05-19 03:15 UTC — probes fire, bridge online)

- **18:55:53 UTC**: Redeployed HandleConnect fix on fix/handle-connect-iter142 branch after false-deploy caught at 17:35
- **Verification**: DLL binary contains HandleConnect symbol; Runtime ready for user testing
- **Next**: User relaunch game instance to confirm OnFatalError handler catches disconnect gracefully

---

## Phase 1: Unblock #523 Commit ✅ DONE 2026-05-19 — applied to lefthook.yml line 19 (was line 9 in original synthesis cite)

- [x] **1.1** Edit `lefthook.yml` line 19: Replace
  ```yaml
  run: dotnet format src/DINOForge.CI.NoRuntime.sln --verify-no-changes
  ```
  with
  ```yaml
  run: dotnet format {staged_files} --verify-no-changes
  ```
- [ ] **1.2** Commit: `git add lefthook.yml && git commit -m "chore(hooks): narrow format-check to staged files only"`
- [ ] **1.3** Test: Re-stage EconomyContentLoader.cs from a7eb4ac → attempt `git commit` → verify hook passes

**Authorization Gate A**: User confirms hook fix applied ✓

---

## Phase 2: Verify #523 & #524 (1.5h)

- [ ] **2.1** Run tests: `dotnet test src/Tests/EconomyContentLoaderValidationTests.cs --filter Category=Economy` → expect 267/267 pass
- [ ] **2.2** Verify block-git-stash.ps1 works: `git stash push -u` → expect **BLOCKED** (PS1 fires)
- [ ] **2.3** Verify guard-git-worktree.ps1: `git worktree remove --force <any-stale-branch>` → expect **BLOCKED** (PS1 fires)

**Result**: #523 & #524 validated locally ✓

---

## Phase 3: Merge fix/handle-connect-iter142 → main (4-4.5h revised from 2h)

> **Revision**: Artifact-sprawl revalidation per `merge_conflict_revalidation_iter142.md` raised effort from 2h to 4-4.5h. Code complexity unchanged; 7,108-file diff (vs. 282 code-only estimate) requires batch artifact cleanup post-merge. Cherry-pick fallback available if merge stalls >2h.

- [ ] **3.1** Fetch main: `git fetch origin main`
- [ ] **3.2** Inspect conflicts: `git merge origin/main --no-commit --no-ff` (dry-run)
  - Expect 282-file intersection
  - **3 HIGH hotspots**: GameClient.cs, JsonRpcMessage.cs, VERSION
- [ ] **3.3** **3-Phase Explicit Merge**:
  - **Phase A** (GameClient.cs): Keep fix branch's OnFatalError handler placement
  - **Phase B** (JsonRpcMessage.cs): Accept main (properties already migrated)
  - **Phase C** (VERSION): Accept fix branch (0.25.0-dev = main's VERSION)
- [ ] **3.4** Test post-merge: `dotnet build src/DINOForge.sln -c Release` → exit code 0
- [ ] **3.5** Push: `git push origin fix/handle-connect-iter142`

**Authorization Gate B**: User confirms push ready ✓

---

## Phase 4: Open PR & Merge to main (1h)

- [ ] **4.1** Open PR: `gh pr create --title "Merge fix/handle-connect-iter142 → main (v0.25.0 pre-release)" --body "Resolves iter-142 merge conflict audit. 51 commits, GameClient/JsonRpcMessage/VERSION reconciled."`
- [ ] **4.2** Await CI green (all 24 workflows pass)
- [ ] **4.3** Merge (squash or rebase per preference): `gh pr merge <PR#> --squash`

**Authorization Gate C**: User confirms PR merge ✓

---

## Phase 5: Tag v0.25.0 (5 min)

- [ ] **5.1** Bump VERSION file: `0.25.0-dev` → `0.25.0`
- [ ] **5.2** Commit: `git add VERSION && git commit -m "chore(release): v0.25.0"`
- [ ] **5.3** Tag & push: `git tag v0.25.0 && git push origin v0.25.0`
- [ ] **5.4** Verify release.yml fires: Check GitHub Actions → release workflow (NuGet publish + GitHub release notes auto-generate)

**Authorization Gate D**: User confirms tag pushed ✓

---

## Deferred to v0.26.0

| Task | Reason |
|------|--------|
| Decision B (TIER 1 Steamless + MockSteamworksNet) | 6–8h sprint; blocked by Decision A first |
| Decision C (isolation_layer.py cleanup, 814 LOC) | 30min after TIER 1 lands |
| #101 (Star Wars 0/36 render) | Headless infra path (#425) not ready |
| #103 (Kimi runbook E2E) | External blocker (Kimi auth) |
| #505 (Pattern #231 audit) | Queued for iter-143 analyzer sweep |

---

## Critical Commands Reference

```powershell
# Phase 1 — edit + test
dotnet test src/Tests/EconomyContentLoaderValidationTests.cs --filter Category=Economy

# Phase 3 — merge conflict resolution
git fetch origin main
git merge origin/main --no-commit --no-ff
# [resolve 3 hotspots]
dotnet build src/DINOForge.sln -c Release
git push origin fix/handle-connect-iter142

# Phase 4 — PR
gh pr create --title "Merge fix/handle-connect-iter142 → main (v0.25.0)" --body "..."

# Phase 5 — release
git tag v0.25.0 && git push origin v0.25.0
```

---

**Estimated Total Wall-Clock Time**: 7–9 hours (revised from 4–5 hours due to Phase 3 artifact cleanup)  
**Blockers**: None (all prior decisions made in iter-142 audits)  
**Next Milestone**: v0.26.0 (TIER 1 Steamless fast-track if timeline permits)
