# Branch Collapse Audit — 2026-06-14

## Method

For each local branch except `main` and `recover/*`, run:

```bash
git cherry origin/main <branch>
```

- If zero lines start with `+` → branch is fully merged residue → `git branch -D <branch>`
- If any `+` lines → branch has unique commits → **SKIP** and record for follow-up

## Results

| Metric | Count |
|--------|-------|
| Total local branches (before) | 180 |
| Total local branches (after) | 96 |
| Branches deleted | 85 |
| Branches with unique commits (skipped) | 79 |
| Worktree-locked branches (zero unique commits) | 12 |
| `recover/*` branches preserved | 3 |

## Skipped Branches (unique commits — needs follow-up recovery)

| Branch | Unique Commits |
|--------|----------------|
| backup/frecon005-20260614 | 498 |
| chore/bevy-omniroute-parallel | 187 |
| chore/dependabot-frontend-2026-06-05 | 426 |
| chore/parallel-session-sync | 115 |
| chore/tech-debt-sweep | 108 |
| civis-pbr | 338 |
| docs/branch-recovery-worklist | 1 |
| docs/p-p1-kickoff | 201 |
| docs/phantom-id-triage-2 | 1 |
| docs/phantom-id-triage-3 | 1 |
| docs/sync-status-2026-05-28 | 201 |
| feat/astar-obstacle-pathfinding | 104 |
| feat/build-next-13 | 1 |
| feat/build-next-7 | 1 |
| feat/civ003-lifecycle | 498 |
| feat/civ007-diplomacy | 486 |
| feat/civis-bevy-game | 247 |
| feat/civis-life-sim | 260 |
| feat/civis-pbr2-triplanar | 346 |
| feat/civis-theme-fix | 347 |
| feat/civis-wave1-emergence | 431 |
| feat/emergence-live-wiring | 1 |
| feat/emergence-onto-main | 10 |
| feat/frecon005-allocation | 498 |
| feat/p-l1-kickoff | 212 |
| feat/p-p1-fr040-geology | 116 |
| feat/p-w1-bevy-gameplay-026 | 190 |
| feat/p-w1-bevy-item-027 | 241 |
| feat/p-w1-civsave-zst | 131 |
| feat/p-w1-float-flow | 131 |
| feat/p-w1-mod-install | 131 |
| feat/p-w1-tactics-002-los | 94 |
| feat/p-w1-tactics-009 | 97 |
| feat/p-w1-tactics-010 | 101 |
| feat/p-w1-tactics-011 | 105 |
| feat/process-compose | 108 |
| feat/session-persistence | 1 |
| feat/tactics-ui | 106 |
| feat/war-bridge-los-formation | 103 |
| fix/clippy-warnings | 176 |
| fix/governance-gate-cache-bypass | 456 |
| fix/justfile-check | 184 |
| fix/launch-asset-sync | 457 |
| fix/pr-333-review | 429 |
| fix/tactics-fog-of-war-wire-in | 104 |
| fix/terrain-fragmentation | 346 |
| merge-333-arena | 431 |
| side/perf-probe | 1 |
| test/fr-batch11 | 1 |
| test/fr-linkage-3 | 1 |
| wip/asset-audit | 474 |
| wip/civ003-design | 476 |
| wip/civ007-design | 473 |
| wip/econ-tiering-pending-verify | 472 |
| wip/gfx-settings | 480 |
| wip/native-ocean | 479 |
| wip/terrain-apron | 478 |
| wip/ui-holocron-theme | 480 |
| wt/actor-y-fix | 357 |
| wt/capability-enforce | 155 |
| wt/chunk-seam | 358 |
| wt/emergence-spawn | 370 |
| wt/map-seed | 371 |
| wt/map2d-ux-2494 | 395 |
| wt/map2d-zoom | 357 |
| wt/mod-hot-reload | 147 |
| wt/mod-publish-store | 145 |
| wt/policy-mod-sdk | 162 |
| wt/remote-mod-store | 155 |
| wt/rust-mod-verify | 130 |
| wt/rust-tests | 133 |
| wt/save-session-db | 154 |
| wt/save-slot-rpc | 145 |
| wt/session-saved-bus | 161 |
| wt/tools-wire | 369 |
| wt/ui-design | 370 |
| wt/water-placement | 357 |
| wt/web-remote-mod-ui | 162 |
| wt/web-save-slot-rpc | 145 |

## Worktree-Locked Branches (zero unique commits — manual cleanup needed)

These branches are fully merged but cannot be deleted because they are currently checked out in linked worktrees:

| Branch |
|--------|
| ci/zero-minutes-hardening |
| docs/coverage-baseline |
| docs/fr-matrix |
| docs/readme-workstate-20260610 |
| feat/audio-substrate |
| feat/emergence-dashboard |
| feat/streaming-window-design |
| feat/verify-harness |
| fix/ci-billing-guard-alert-sync |
| fix/reusable-caller-permissions |
| fix/terrain-fragmentation-ship |
| perf/frame-baseline-rerun |

## Action

1. **Skipped branches** — Review for cherry-pick, rebase, or merge into main.
2. **Locked branches** — Close linked worktrees, then re-run `git branch -D`.
3. **Deleted branches** — Resurrect via `git reflog` if needed within 30 days.
