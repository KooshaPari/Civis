# Branch Sweep Classification — 2026-06-11

## Summary

36 local branches. **None are cleanly PR-able to main** — every local branch's
branch-point predates main's large growth (each shows ~1.26–1.33M *deletions* when
diffed against main, i.e. main has added that much since they forked). A naive
`PR branch → main` would attempt to delete the bulk of current main. The correct
sweep outcome is **classify + archive/delete**, NOT mass-PR.

## Classification

| Category | Count | Branches | Disposition |
|---|---|---|---|
| **reconcile-3 era feat/fix** (dated 2026-05-30 → 2026-06-01) | 16 | `feat/env-theme-swap-20260531`, `feat/naval-20260531`, `feat/loadingscreen-20260531`, `feat/modern-20260531`, `feat/icons-20260601`, `feat/cursors-20260601`, `feat/bldicons-20260531`, `feat/uicensus-20260601`, `feat/rigging-optimizer-20260531`, `feat/sw-building-bundles-20260531`, `fix/assetswap-runtime-bugs-20260530`, `fix/sw-tmp-font-20260531`, `fix/cursor-agent-config-resolution-20260531`, `epic027-catalog-20260530`, `docs/full-world-sw-plan-20260530`, `rnd/brickalyzer-20260531` | **SUPERSEDED** — feature content already merged to main via reconcile-3 (tasks #985/#987 closed). Safe to delete. |
| **snyk-fix** (security) | 3 | `snyk-fix-7389a3c…`, `snyk-fix-80168d90…`, `snyk-fix-9edfd94c…` | Ancient branch-point. If the dependency bumps are still relevant, **re-apply fresh against current main**; the branches themselves are not mergeable. |
| **stash/recovered** | 2 | `stash/recovered-2026-05-19-1`, `stash/recovered-2026-05-19-2` | Recovery snapshots — archival only, not for PR. |
| **safety/iter snapshots** | 2 | `safety/iter140-snapshot-2026-05-18`, `safety/iter145-recovery-20260523-0432` | Point-in-time safety snapshots — archival only. |
| **misc/infra** | ~2 | `agent/coderabbit-main-config`, `docs/ci-workflow-bootstrap` | Re-evaluate individually; likely superseded. |
| **active** | 1 | `wsm/agileplus-dag-20260610` | Current PR #279 branch — keep. |

## Recommended action

- **Delete** the 16 reconcile-3-era branches + 2 stash + 2 safety (all superseded/archival).
  Deletion deferred: the AgilePlus tree is swarmed and the repo has concurrent sessions;
  per the no-git-branch-ops-while-agents-run rule, branch deletion should be done in a
  quiet window or via a dedicated worktree, not from the orchestrator mid-swarm.
- **Re-apply** snyk dependency fixes fresh against main if still flagged by Snyk.
- The "35 unique branches → PR each" framing in the cron is **not actionable as written** —
  the branches are stale, not parallel work streams. Zero-limbo is achieved by deletion,
  not by 35 destructive PRs.

## Why not PR them

Each branch diff vs main is dominated by reverse-deletions (main's own additions since
fork). Example: `feat/env-theme-swap-20260531` → main = 695 files, +2307 / −1,260,623.
The +2307 is the branch's original feature delta (already in main via reconcile-3); the
−1.26M is everything main gained afterward. A merge/PR would regress main. Confirmed the
features are live on main via the closed tasks for each (#985 skybox, #987 reconcile-3,
#992 bundle pipeline, etc.).
