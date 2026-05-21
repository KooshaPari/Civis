# Iter-142 Consolidation Readiness Checklist

**Generated**: 2026-05-18  
**Branch**: fix/handle-connect-iter142 @ 411e34b8  
**v0.25.0 target**

## Pre-Push Gates

- [x] HandleConnect implemented + tested + deployed to game install (commit ced0dccf)
- [x] iter-142 audit + governance docs committed (commit 411e34b8)
- [x] Safety branch on remote (durable backup): safety/iter140-snapshot-2026-05-18 @ f699154e
- [x] PreToolUse hooks wired (block-git-stash + guard-git-worktree)
- [x] 3 durable feedback memories in MEMORY.md
- [ ] **Closure-gate GREEN on fix branch** — currently YELLOW with 9 EconomyContentLoader regressions (#523 in flight)
- [ ] **User authorizes push** of fix/handle-connect-iter142 to origin

## Pre-Merge Gates (after push)

- [ ] `git fetch origin --all`
- [ ] Merge origin/main into fix/handle-connect-iter142 (282 file conflict surface, 4 HIGH hotspots: GameClient.cs, JsonRpcMessage.cs, GameBridgeServer.cs, VERSION)
- [ ] Manual conflict resolution (estimated HIGH effort, 1–3h focused)
- [ ] Rebuild + re-run closure-gate post-merge
- [ ] Tests pass on merged state

## Pre-PR Gates

- [ ] Push merged branch to origin
- [ ] Open PR via `gh pr create` using docs/sessions/CONSOLIDATION_PR_DESCRIPTION_iter142.md
- [ ] PR description references release notes, playbook, retrospective

## Pre-Merge-to-Main Gates

- [ ] User reviews + approves PR
- [ ] `gh pr merge --merge` (or --squash if cleaner)
- [ ] Verify main updated on remote
- [ ] Pull main locally

## Post-Merge Cleanup

- [ ] Delete 7 stale branches per docs/sessions/branch_deletion_plan_iter142.md (Batch 1)
- [ ] Delete safety/iter140-snapshot-2026-05-18 (Batch 3, after iter-142 verified merged)
- [ ] Close GH issue #129 (Rust pipeline — WONTFIX)
- [ ] Comment on issues #130, #131 (defer to v0.26.0 coverage sprint)

## v0.25.0 Tag (separate authorization)

- [ ] `git tag -a v0.25.0 -m "v0.25.0 — Tier 2 Roslyn x27, Tier 3 fuzz x162, Pattern Catalog x30"`
- [ ] `git push origin v0.25.0`
- [ ] Verify release.yml triggers (NuGet publish + GitHub Release Draft)

## Post-Tag

- [ ] Bump VERSION to 0.26.0-dev
- [ ] Open [0.26.0-dev] CHANGELOG section
- [ ] Schedule v0.26.0 work per docs/releases/v0.26.0-FORWARD-PLAN.md

## Rollback Plans

| Scenario | Command | Notes |
|----------|---------|-------|
| Push rollback | Branch on remote preserved | Delete safety snapshot only after merged |
| Merge rollback | `git reset --hard 411e34b8` (local) | Pre-merge SHA preserved |
| Main merge rollback | Revert merge commit via PR | Preserves history |
| Tag rollback | `git tag -d v0.25.0 && git push origin :v0.25.0` | Caution: release.yml may trigger |

## Reference Documents

- **Release notes**: docs/releases/v0.25.0-RELEASE-NOTES.md
- **v0.26.0 plan**: docs/releases/v0.26.0-FORWARD-PLAN.md
- **Playbook**: docs/sessions/branch_consolidation_playbook_iter142.md
- **PR description**: docs/sessions/CONSOLIDATION_PR_DESCRIPTION_iter142.md
- **Retrospective**: docs/sessions/iter-142-retrospective.md
- **Merge conflict prediction**: docs/sessions/merge_conflict_prediction_iter142.md
- **Branch deletion plan**: docs/sessions/branch_deletion_plan_iter142.md
- **Pattern catalog**: docs/qa/PATTERN_CATALOG_v0.25.0_SUMMARY.md
- **TRUTH_TABLE**: Root of repo (3,222 lines, all data gates + test coverage)
