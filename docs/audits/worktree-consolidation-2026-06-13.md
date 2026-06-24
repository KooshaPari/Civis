# Worktree Consolidation Audit

Date: 2026-06-13
Total worktrees: 97

## Methodology

For each worktree we report:

- **Branch**: The checked-out branch (or `(detached)` for detached HEAD).
- **Ahead**: Commits on the branch that are not ancestors of `origin/main`.
- **Behind**: Commits on `origin/main` that are not ancestors of the branch.
- **Cherry Unique**: Number of `+` lines from `git cherry origin/main <branch>` — patches *not* present in `origin/main`.
- **Cherry Duplicate**: Number of `-` lines from `git cherry origin/main <branch>` — patches already present in `origin/main`.
- **Locked**: Whether the worktree is locked by `git worktree` (usually indicates an active process or IDE).

### Classification Rules

| Classification | Criteria |
|---|---|
| **SAFE-TO-REMOVE** | Not locked, and `cherry_unique = 0` (all patches already in `origin/main`). Detached HEADs whose commit patch is already in `origin/main` also fall here. |
| **HAS-UNIQUE-CODE** | Not locked, and `cherry_unique > 0` (contains commits whose patches are *not* in `origin/main`). |
| **LOCKED** | The worktree is locked by `git worktree`. Locked worktrees must be unlocked before removal. Some locked worktrees also contain unique code; this is noted in the remarks. |

---

## Summary

| Classification | Count |
|---|---|
| SAFE-TO-REMOVE | 83 |
| HAS-UNIQUE-CODE | 10 |
| LOCKED | 4 |

---

## Full Worktree Register

| # | Path | Branch | Commit | Ahead | Behind | Cherry Unique | Cherry Duplicate | Locked | Classification | Remarks |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | `C:/Users/koosh/Dev/Civis` | `side/deadcode-prune` | `de5b2803` | 0 | 0 | 0 | 0 | no | **SAFE-TO-REMOVE** | Primary worktree. Same commit as `origin/main`. |
| 2 | `C:/Users/koosh/Dev/civis-game` | `feat/frecon005-allocation` | `1c7ab3ab` | 548 | 114 | 498 | 30 | no | **HAS-UNIQUE-CODE** | Large feature branch with 498 unique commits. |
| 3 | `C:/Users/koosh/Dev/civis-wt-readme3` | `docs/readme-workstate-20260610` | `4bb0dfe3` | 1 | 84 | 0 | 1 | **yes** | **LOCKED** | Locked. 0 unique commits. |
| 4 | `C:/Users/koosh/Dev/civis-wt-trace-2026-06-10` | `docs/traceability-20260610` | `197b0ed8` | 1 | 105 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 5 | `D:/civis-build/batch10` | `test/fr-batch10` | `c6f58a71` | 1 | 34 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 6 | `D:/civis-build/batch9` | `test/fr-batch9` | `85c5d9b9` | 1 | 35 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 7 | `D:/civis-build/bn1` | `feat/build-next-1` | `4513b2ef` | 1 | 46 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 8 | `D:/civis-build/bn10` | `feat/build-next-10` | `1e634b65` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 9 | `D:/civis-build/bn11` | `feat/build-next-11` | `04a3fbb3` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 10 | `D:/civis-build/bn12` | `feat/build-next-12` | `490585a3` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 11 | `D:/civis-build/bn13` | `feat/build-next-13` | `369eabaa` | 1 | 27 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 12 | `D:/civis-build/bn14` | `feat/build-next-14` | `ba63d67b` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 13 | `D:/civis-build/bn15` | `feat/build-next-15` | `23f2dd1c` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 14 | `D:/civis-build/bn2` | `feat/build-next-2` | `85a3e244` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 15 | `D:/civis-build/bn3` | `feat/build-next-3` | `9ce8ee8b` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 16 | `D:/civis-build/bn4` | `feat/build-next-4` | `546d6465` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 17 | `D:/civis-build/bn5` | `feat/build-next-5` | `25f059ea` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 18 | `D:/civis-build/bn6` | `feat/build-next-6` | `0666f327` | 1 | 34 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 19 | `D:/civis-build/bn7` | `feat/build-next-7` | `9bf50d18` | 1 | 34 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 20 | `D:/civis-build/bn8` | `feat/build-next-8` | `5558ca98` | 1 | 34 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 21 | `D:/civis-build/bn9` | `feat/build-next-9` | `10e7ce10` | 1 | 28 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 22 | `D:/civis-build/cov11` | `test/fr-batch11` | `552916ed` | 1 | 11 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 23 | `D:/civis-build/cov12` | `test/fr-batch12` | `63384f1c` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 24 | `D:/civis-build/cov13` | `test/fr-batch13` | `b950c019` | 0 | 12 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 25 | `D:/civis-build/defer1` | `feat/defer-promote-1` | `b843015d` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 26 | `D:/civis-build/defer2` | `feat/defer-promote-2` | `1f8f6f4e` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 27 | `D:/civis-build/demands` | `docs/user-demand-trace` | `4dc70111` | 1 | 76 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 28 | `D:/civis-build/diaginst` | `fix/standalone-diagnostics` | `af8463de` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 29 | `D:/civis-build/emsource` | `fix/emergence-sampling-source` | `60a7d725` | 1 | 49 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 30 | `D:/civis-build/flaky` | `fix/autosave-test-race` | `424a5f91` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 31 | `D:/civis-build/fr-save` | `test/fr-save-epic` | `07c93587` | 0 | 10 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 32 | `D:/civis-build/fr-ux` | `test/fr-ux-epic` | `07c93587` | 0 | 10 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 33 | `D:/civis-build/linkage` | `test/fr-linkage-1` | `5e7156ce` | 1 | 72 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 34 | `D:/civis-build/linkage2` | `test/fr-linkage-2` | `ccfbef64` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 35 | `D:/civis-build/linkage3` | `test/fr-linkage-3` | `7563176f` | 1 | 67 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 36 | `D:/civis-build/linkrec` | `test/fr-linkage-recovered` | `8b8459ff` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 37 | `D:/civis-build/matrix2` | `docs/fr-matrix-rerun` | `40102a21` | 1 | 46 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 38 | `D:/civis-build/matrix3` | `docs/fr-matrix-rerun-2` | `219d76a4` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 39 | `D:/civis-build/matrix4` | `docs/fr-matrix-rerun-3` | `50888a7a` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 40 | `D:/civis-build/matrix5` | `docs/fr-matrix-rerun-4` | `5b008af0` | 1 | 17 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 41 | `D:/civis-build/nfr-perf` | `test/nfr-perf-epic` | `07c93587` | 0 | 10 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 42 | `D:/civis-build/parity` | `docs/parity-benchmark` | `76c86ece` | 1 | 77 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 43 | `D:/civis-build/parity1` | `feat/parity-pbr-infra-1` | `a580f4a9` | 1 | 13 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 44 | `D:/civis-build/phantom-codify` | `chore/phantom-codify` | `07c93587` | 0 | 10 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 45 | `D:/civis-build/rb-348` | `(detached)` | `4bb0dfe3` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `4bb0dfe3`. `git cherry` confirms patch is already in `origin/main`. |
| 46 | `D:/civis-build/rb-355` | `chore/hygiene-20260610` | `2b234ddb` | 1 | 84 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 47 | `D:/civis-build/rb-357` | `main` | `d627d336` | 0 | 0 | 0 | 0 | no | **SAFE-TO-REMOVE** | On `main`. |
| 48 | `D:/civis-build/rb-366` | `(detached)` | `43db79bc` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `43db79bc`. `git cherry` confirms patch is already in `origin/main`. |
| 49 | `D:/civis-build/readme2` | `docs/readme-workstate-2` | `1063cebf` | 1 | 27 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 50 | `D:/civis-build/scanfix` | `fix/matrix-scanner-covers` | `84e686a4` | 1 | 35 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 51 | `D:/civis-build/speconly` | `docs/spec-only-triage-1` | `3ade5dd1` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 52 | `D:/civis-build/speconly2` | `docs/spec-only-triage-2` | `18ea5b32` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 53 | `D:/civis-build/speconly3` | `docs/spec-only-triage-3` | `3a7dc7d5` | 1 | 53 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 54 | `D:/civis-build/stale1` | `chore/stale-id-sweep-1` | `d5c56ea4` | 1 | 46 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 55 | `D:/civis-build/stale2` | `chore/stale-id-sweep-2` | `9c646126` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 56 | `D:/civis-build/tactics-hud` | `test/tactics-hud-epic` | `07c93587` | 0 | 10 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 57 | `D:/civis-build/tests1` | `test/fr-batch1` | `cfe48e30` | 1 | 77 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 58 | `D:/civis-build/tests2` | `test/fr-batch2` | `f90dc0a7` | 1 | 72 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 59 | `D:/civis-build/tests3` | `test/fr-batch3` | `efe5961d` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 60 | `D:/civis-build/tests4` | `test/fr-batch4` | `a26006a5` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 61 | `D:/civis-build/tests5` | `test/fr-batch5` | `cbeaa5ec` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 62 | `D:/civis-build/tests6` | `test/fr-batch6` | `bc29b618` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 63 | `D:/civis-build/tests7` | `test/fr-batch7` | `a440a13f` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 64 | `D:/civis-build/tests8` | `test/fr-batch8` | `a416b634` | 1 | 45 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 65 | `D:/civis-build/triage` | `docs/phantom-id-triage` | `7b15da69` | 1 | 81 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 66 | `D:/civis-build/triage2` | `docs/phantom-id-triage-2` | `7368d0ca` | 1 | 74 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 67 | `D:/civis-build/triage3` | `docs/phantom-id-triage-3` | `c2ab0435` | 2 | 67 | 1 | 1 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 68 | `D:/civis-build/triage4` | `docs/phantom-id-triage-4` | `16f0918d` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 69 | `D:/civis-build/triage5` | `docs/phantom-id-triage-5` | `bf26482b` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 70 | `D:/civis-build/triage6` | `docs/phantom-id-triage-6` | `ee58c795` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 71 | `D:/civis-build/triage7` | `docs/phantom-id-triage-7` | `35fff271` | 1 | 67 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 72 | `D:/civis-build/triage8` | `docs/phantom-id-triage-8` | `e8b0ad90` | 1 | 53 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 73 | `D:/civis-build/triage9` | `docs/phantom-id-triage-9` | `482ffaa7` | 1 | 53 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 74 | `D:/civis-build/warepic` | `test/war-build-bevy-epic` | `d7f022df` | 1 | 13 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 75 | `D:/civis-build/warnfix` | `fix/warnings-as-errors` | `2aeb4157` | 1 | 76 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 76 | `D:/civis-build/wt-frame-baseline` | `perf/frame-baseline` | `03c3c2f7` | 1 | 79 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 77 | `E:/civis-merge333` | `merge-333-arena` | `8d2c8c17` | 477 | 114 | 431 | 28 | **yes** | **LOCKED** | Locked. 431 unique commits (same as `feat/civis-wave1-emergence`). |
| 78 | `E:/civis-wt-assetroot2` | `fix/asset-root-fallback` | `af913fb2` | 0 | 105 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 79 | `E:/civis-wt-ca-dirty` | `(detached)` | `a8b19bdb` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `a8b19bdb`. Commit is an ancestor of `origin/main`. |
| 80 | `E:/civis-wt-ca-dirty-tmp` | `(detached)` | `a8b19bdb` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `a8b19bdb`. Commit is an ancestor of `origin/main`. |
| 81 | `E:/civis-wt-ci-billing-guard-fresh` | `fix/ci-billing-guard-alert-sync` | `a8b19bdb` | 0 | 106 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 82 | `E:/civis-wt-depsweep-fresh` | `(detached)` | `a8b19bdb` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `a8b19bdb`. Commit is an ancestor of `origin/main`. |
| 83 | `E:/civis-wt-emergence-dash` | `feat/emergence-dashboard` | `a8b19bdb` | 0 | 106 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 84 | `E:/civis-wt-m333-fresh` | `feat/civis-wave1-emergence` | `8d2c8c17` | 477 | 114 | 431 | 28 | no | **HAS-UNIQUE-CODE** | Same branch/commit as `merge-333-arena` (E:/civis-merge333). 431 unique commits. |
| 85 | `E:/civis-wt-readme4` | `docs/readme-workstate-20260610b` | `00000000` | N/A | N/A | N/A | N/A | **yes** | **LOCKED** | Locked. Empty checkout (`00000000`). |
| 86 | `E:/civis-wt-reusable-perms` | `fix/reusable-caller-permissions` | `a8b19bdb` | 0 | 106 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 87 | `E:/civis-wt-terrain-ship` | `fix/terrain-fragmentation-ship` | `a8b19bdb` | 0 | 106 | 0 | 0 | **yes** | **LOCKED** | Locked. 0 unique commits. |
| 88 | `E:/civis-wt-verify` | `feat/verify-harness` | `a8b19bdb` | 0 | 106 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 89 | `E:/civis-wt-verify-2` | `(detached)` | `a8b19bdb` | 0 | N/A | N/A | N/A | no | **SAFE-TO-REMOVE** | Detached at `a8b19bdb`. Commit is an ancestor of `origin/main`. |
| 90 | `G:/civis-main-gate` | `perf/frame-baseline-rerun` | `a91d2976` | 1 | 72 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 91 | `G:/civis-wt-audio` | `feat/audio-substrate` | `af913fb2` | 0 | 105 | 0 | 0 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 92 | `G:/civis-wt-cifix` | `ci/zero-minutes-hardening` | `55f32c07` | 1 | 100 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 93 | `G:/civis-wt-cov` | `docs/coverage-baseline` | `5b843185` | 1 | 105 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 94 | `G:/civis-wt-emwire` | `feat/emergence-live-wiring` | `5cc53c78` | 2 | 105 | 1 | 1 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 95 | `G:/civis-wt-matrix` | `docs/fr-matrix` | `4529a809` | 1 | 105 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |
| 96 | `G:/civis-wt-persist` | `feat/session-persistence` | `9974dce8` | 1 | 105 | 1 | 0 | no | **HAS-UNIQUE-CODE** | 1 unique commit. |
| 97 | `G:/civis-wt-scale` | `feat/streaming-window-design` | `1fca3c9c` | 1 | 105 | 0 | 1 | no | **SAFE-TO-REMOVE** | 0 unique commits. |

---

## Detailed Findings

### SAFE-TO-REMOVE (83 worktrees)

These worktrees contain no patches that are absent from `origin/main`. They can be removed with `git worktree remove <path>` (or `git worktree remove --force <path>` if there are untracked files). They do not need to be committed, pushed, or merged.

Notable sub-groups:

- **Primary worktree**: `C:/Users/koosh/Dev/Civis` is on `side/deadcode-prune` at the exact same commit as `origin/main` (`de5b2803`). This worktree cannot be removed because it is the main repository, but the branch itself can be deleted if desired.
- **On `main`**: `D:/civis-build/rb-357` is directly on `main`.
- **Detached HEADs already in main**: `D:/civis-build/rb-348` (`4bb0dfe3`), `D:/civis-build/rb-366` (`43db79bc`), and all worktrees at `a8b19bdb` (E:/civis-wt-ca-dirty, E:/civis-wt-ca-dirty-tmp, E:/civis-wt-depsweep-fresh, E:/civis-wt-verify-2) are detached at commits whose patches are already present in `origin/main`.
- **Single-commit branches with duplicate patches**: The vast majority of test/docs branches (`bn1` through `bn15`, `batch9`, `batch10`, `tests1` through `tests8`, `triage` through `triage9`, etc.) have `ahead=1` and `cherry_duplicate=1`, meaning their single commit is already in `origin/main`.
- **Zero-ahead branches**: `cov13`, `fr-save`, `fr-ux`, `nfr-perf`, `phantom-codify`, `tactics-hud`, `civis-wt-assetroot2`, `civis-wt-ci-billing-guard-fresh`, `civis-wt-emergence-dash`, `civis-wt-reusable-perms`, `civis-wt-verify`, `civis-wt-audio` are all exactly at a commit already in `origin/main` (no unique commits, but behind main).

### HAS-UNIQUE-CODE (10 worktrees)

These worktrees contain at least one commit whose patch is **not** present in `origin/main`. They should be reviewed before removal. If the code is still valuable, the branch should be committed, pushed, a draft PR opened, and then squash-merged to `main` (or the branch preserved if the work is ongoing).

| Path | Branch | Unique Commits | Notes |
|---|---|---|---|
| `C:/Users/koosh/Dev/civis-game` | `feat/frecon005-allocation` | 498 | Large feature branch. Significant divergence from main (114 behind). |
| `D:/civis-build/bn13` | `feat/build-next-13` | 1 | 1 unique commit. |
| `D:/civis-build/bn7` | `feat/build-next-7` | 1 | 1 unique commit. |
| `D:/civis-build/cov11` | `test/fr-batch11` | 1 | 1 unique commit. |
| `D:/civis-build/linkage3` | `test/fr-linkage-3` | 1 | 1 unique commit. |
| `D:/civis-build/triage2` | `docs/phantom-id-triage-2` | 1 | 1 unique commit. |
| `D:/civis-build/triage3` | `docs/phantom-id-triage-3` | 1 | 1 unique commit. |
| `E:/civis-wt-m333-fresh` | `feat/civis-wave1-emergence` | 431 | Same commit as `merge-333-arena`. 431 unique commits. |
| `G:/civis-wt-emwire` | `feat/emergence-live-wiring` | 1 | 1 unique commit. |
| `G:/civis-wt-persist` | `feat/session-persistence` | 1 | 1 unique commit. |

### LOCKED (4 worktrees)

These worktrees are locked by `git worktree`. A locked worktree usually means a process is holding it open (e.g., an IDE, build, or long-running shell). You must run `git worktree unlock <path>` before removal.

| Path | Branch | Locked Reason (inferred) | Unique Code | Notes |
|---|---|---|---|---|
| `C:/Users/koosh/Dev/civis-wt-readme3` | `docs/readme-workstate-20260610` | Locked | No | 0 unique commits. Safe to remove after unlocking. |
| `E:/civis-merge333` | `merge-333-arena` | Locked | Yes (431) | Contains 431 unique commits (same as `feat/civis-wave1-emergence`). Must be unlocked before removal. |
| `E:/civis-wt-readme4` | `docs/readme-workstate-20260610b` | Locked | N/A | Empty checkout (`00000000`). Safe to remove after unlocking. |
| `E:/civis-wt-terrain-ship` | `fix/terrain-fragmentation-ship` | Locked | No | 0 unique commits. Safe to remove after unlocking. |

---

## Consolidation Commands (Reference)

**Do NOT run these automatically.** The user has requested an audit only. The commands below are provided for reference when the user decides to act.

### SAFE-TO-REMOVE worktrees

```bash
# Remove a single worktree
git worktree remove <path>

# Remove all SAFE-TO-REMOVE worktrees (except the primary)
# This would remove 82 worktrees (excluding C:/Users/koosh/Dev/Civis)
```

### HAS-UNIQUE-CODE worktrees

For each, the typical workflow is:

```bash
# 1. Ensure all changes are committed
# 2. Push the branch
git push origin <branch>

# 3. Open a draft PR via GitHub CLI
gh pr create --draft --title "<branch>" --body "..."

# 4. After review, squash-merge and delete branch
gh pr merge <pr-number> --squash --delete-branch

# 5. Remove the worktree
git worktree remove <path>
```

### LOCKED worktrees

```bash
# 1. Unlock
git worktree unlock <path>

# 2. Then proceed based on classification (SAFE-TO-REMOVE or HAS-UNIQUE-CODE)
```

---

## Disposition

- **No worktrees were removed, committed, pushed, or merged in this audit.**
- **No PRs were created.**
- **No branches were deleted.**

This document is a read-only classification for manual review.
