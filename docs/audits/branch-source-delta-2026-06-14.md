# Branch Source-Delta Audit - 2026-06-14

**Method:** For each branch with unique commits, count lines of `git diff origin/main..BRANCH`
restricted to actual source files (`crates/**/*.rs`, `clients/**/*.rs`, `web/src/**`).
Branches with >50 source-delta lines are flagged as **NEEDS-REVIEW**; the rest are negligible.

| Branch | Source-Delta-Lines | Verdict |
|--------|-------------------:|---------|
| feat/p-w1-tactics-002-los | 84265 | NEEDS-REVIEW |
| feat/p-w1-tactics-009 | 83485 | NEEDS-REVIEW |
| feat/p-w1-tactics-010 | 82105 | NEEDS-REVIEW |
| feat/war-bridge-los-formation | 81707 | NEEDS-REVIEW |
| feat/astar-obstacle-pathfinding | 81488 | NEEDS-REVIEW |
| feat/p-w1-tactics-011 | 81133 | NEEDS-REVIEW |
| feat/tactics-ui | 81133 | NEEDS-REVIEW |
| fix/tactics-fog-of-war-wire-in | 81133 | NEEDS-REVIEW |
| chore/tech-debt-sweep | 79577 | NEEDS-REVIEW |
| feat/process-compose | 79577 | NEEDS-REVIEW |
| chore/parallel-session-sync | 79161 | NEEDS-REVIEW |
| wt/mod-publish-store | 78729 | NEEDS-REVIEW |
| wt/web-save-slot-rpc | 78460 | NEEDS-REVIEW |
| feat/p-w1-mod-install | 78246 | NEEDS-REVIEW |
| wt/rust-mod-verify | 78048 | NEEDS-REVIEW |
| wt/capability-enforce | 78020 | NEEDS-REVIEW |
| wt/remote-mod-store | 78020 | NEEDS-REVIEW |
| feat/p-w1-civsave-zst | 77944 | NEEDS-REVIEW |
| wt/mod-hot-reload | 77943 | NEEDS-REVIEW |
| wt/save-session-db | 77896 | NEEDS-REVIEW |
| feat/p-w1-float-flow | 77884 | NEEDS-REVIEW |
| wt/save-slot-rpc | 77880 | NEEDS-REVIEW |
| wt/rust-tests | 77861 | NEEDS-REVIEW |
| wt/session-saved-bus | 77727 | NEEDS-REVIEW |
| wt/web-remote-mod-ui | 77684 | NEEDS-REVIEW |
| wt/policy-mod-sdk | 77335 | NEEDS-REVIEW |
| fix/clippy-warnings | 76094 | NEEDS-REVIEW |
| fix/justfile-check | 68962 | NEEDS-REVIEW |
| chore/bevy-omniroute-parallel | 66045 | NEEDS-REVIEW |
| feat/p-w1-bevy-gameplay-026 | 66012 | NEEDS-REVIEW |
| docs/p-p1-kickoff | 65992 | NEEDS-REVIEW |
| docs/sync-status-2026-05-28 | 65918 | NEEDS-REVIEW |
| feat/p-p1-fr040-geology | 65839 | NEEDS-REVIEW |
| feat/p-l1-kickoff | 64528 | NEEDS-REVIEW |
| feat/civis-bevy-game | 58798 | NEEDS-REVIEW |
| feat/civis-life-sim | 57562 | NEEDS-REVIEW |
| feat/p-w1-bevy-item-027 | 56572 | NEEDS-REVIEW |
| civis-pbr | 46329 | NEEDS-REVIEW |
| fix/terrain-fragmentation | 43823 | NEEDS-REVIEW |
| feat/civis-theme-fix | 43677 | NEEDS-REVIEW |
| feat/civis-pbr2-triplanar | 43653 | NEEDS-REVIEW |
| wt/actor-y-fix | 42093 | NEEDS-REVIEW |
| wt/map2d-zoom | 42093 | NEEDS-REVIEW |
| wt/water-placement | 42093 | NEEDS-REVIEW |
| wt/chunk-seam | 42081 | NEEDS-REVIEW |
| wt/emergence-spawn | 41001 | NEEDS-REVIEW |
| wt/ui-design | 41001 | NEEDS-REVIEW |
| backup/frecon005-20260614 | 40990 | NEEDS-REVIEW |
| feat/civ003-lifecycle | 40990 | NEEDS-REVIEW |
| feat/frecon005-allocation | 40990 | NEEDS-REVIEW |
| wt/tools-wire | 40977 | NEEDS-REVIEW |
| wt/map-seed | 40972 | NEEDS-REVIEW |
| wt/map2d-ux-2494 | 39884 | NEEDS-REVIEW |
| feat/civ007-diplomacy | 39736 | NEEDS-REVIEW |
| wip/gfx-settings | 39090 | NEEDS-REVIEW |
| wip/ui-holocron-theme | 39090 | NEEDS-REVIEW |
| wip/terrain-apron | 38824 | NEEDS-REVIEW |
| wip/native-ocean | 37805 | NEEDS-REVIEW |
| wip/asset-audit | 37561 | NEEDS-REVIEW |
| wip/civ003-design | 37561 | NEEDS-REVIEW |
| wip/civ007-design | 37561 | NEEDS-REVIEW |
| wip/econ-tiering-pending-verify | 37539 | NEEDS-REVIEW |
| feat/civis-wave1-emergence | 36794 | NEEDS-REVIEW |
| merge-333-arena | 36794 | NEEDS-REVIEW |
| fix/governance-gate-cache-bypass | 36608 | NEEDS-REVIEW |
| fix/launch-asset-sync | 36608 | NEEDS-REVIEW |
| fix/pr-333-review | 36534 | NEEDS-REVIEW |
| chore/dependabot-frontend-2026-06-05 | 36403 | NEEDS-REVIEW |
| feat/session-persistence | 35541 | NEEDS-REVIEW |
| feat/emergence-live-wiring | 34736 | NEEDS-REVIEW |
| docs/phantom-id-triage-2 | 27576 | NEEDS-REVIEW |
| docs/phantom-id-triage-3 | 27130 | NEEDS-REVIEW |
| test/fr-linkage-3 | 27018 | NEEDS-REVIEW |
| feat/build-next-7 | 22399 | NEEDS-REVIEW |
| feat/build-next-13 | 17809 | NEEDS-REVIEW |
| feat/emergence-onto-main | 13561 | NEEDS-REVIEW |
| side/perf-probe | 10290 | NEEDS-REVIEW |
| docs/branch-recovery-worklist | 10238 | NEEDS-REVIEW |
| test/fr-batch11 | 10238 | NEEDS-REVIEW |

**Summary:** 79 of 79 branches have >50 source-delta lines.

*Only the top entries with real source deltas matter; fork-skew/docs branches dominate the long tail.*
