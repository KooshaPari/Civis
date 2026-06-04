# Branch Consolidation Plan — 2026-06-03

## Goal

Consolidate ~57 outstanding feature branches into `main` via **one** consolidation PR
rather than 57 individual PRs. Analysis shows `reconcile3/converge-20260531` is a
**superset** that already contains the work of 46 of the feature branches.

## Strategy

- **Push** `reconcile3/converge-20260531` to remote.
- **Open ONE consolidation PR** (`reconcile3 → main`) as the main consolidation vehicle.
- Layer the two PR-bearing branches on top afterward.
- **Do NOT delete any branches** (cursor-bot or otherwise).
- **Do NOT** use `--no-verify` / bypass hooks. One push at a time (bin/obj lock contention).

## Superset branch

| Branch | Tip | Ahead of `main` |
|---|---|---|
| `reconcile3/converge-20260531` | `d5bc56db` diag(assetswap): per-failure RESOLVE-FAIL logging | 231 commits |

Contains the converged work of 46 feature branches.

## Open PRs

| PR | Head branch | Contents |
|---|---|---|
| #242 | `integration/v0.27.0` | TAKEOVER (menu/subpage takeover work) |
| #243 | `fix/modern-loading-bar-20260531` | iter-153 session work (SW bundles, font fix, loading bar, TAKEOVER conflict resolution) |

## Branches with unique commit tails (need individual review)

These local feature branches carry commits **not** reachable from
`reconcile3/converge-20260531`. Most are 1-commit "salvage pre-capacity-death" WIP
tips; a few carry substantive fixes that should be cherry-picked or PR'd separately.

| Branch | Unique commits | Notable tip |
|---|---|---|
| `fix/modern-loading-bar-20260531` | 9 | PR #243 — iter-153 session (SW bundles, font, loading-bar, TAKEOVER merge) |
| `fix/assetswap-runtime-bugs-20260530` | 4 | **THE 'units look native' fix** — mesh-substring filter rejecting all entities (#101) |
| `feat/rigging-optimizer-20260531` | 3 | #991 blender rig+decimate scaffold, skinned-mesh compat gate |
| `docs/ci-workflow-bootstrap` | 2 | CI workflow bootstrap docs + full ci.yml |
| `feat/loadingscreen-20260531` | 2 | native-symbol gap doc + salvage WIP |
| `fix/sw-tmp-font-20260531` | 2 | #965 offline TMP bake + prebuilt font loading |
| `docs/full-world-sw-plan-20260530` | 1 | full-world SW total-conversion phased plan |
| `feat/bldicons-20260531` | 1 | salvage pre-capacity WIP |
| `feat/cursors-20260601` | 1 | salvage pre-capacity WIP |
| `feat/env-theme-swap-20260531` | 1 | #975 EnvironmentThemeSwap spike |
| `feat/icons-20260601` | 1 | salvage pre-capacity WIP |
| `feat/modern-20260531` | 1 | salvage pre-capacity-death WIP |
| `feat/naval-20260531` | 1 | salvage pre-capacity-death WIP |
| `feat/naval-content-20260531` | 1 | generate remaining stub SW building meshes + proofs (`323b426b`) |
| `feat/sw-building-bundles-20260531` | 1 | rebuild unit bundles with URP shaders |
| `feat/sw-mesh-grind-20260530` | 1 | stub SW building meshes + proofs (`323b426b`, shared) |
| `feat/uicensus-20260601` | 1 | game_dump_ui_tree alias + ui-tree census doc |
| `fix/cursor-agent-config-resolution-20260531` | 1 | salvage pre-capacity-death WIP |
| `rnd/brickalyzer-20260531` | 1 | brick-mode feasibility + prototype |

### Review priorities

1. **`fix/assetswap-runtime-bugs-20260530`** (4 commits) — contains the #101 "units look
   native" root-cause fix. Confirm whether reconcile3's assetswap path already includes
   the mesh-substring filter fix; if not, this MUST be PR'd / cherry-picked.
2. **`feat/rigging-optimizer-20260531`** (3 commits) — #991 substantive scaffold.
3. **`fix/sw-tmp-font-20260531`** (2 commits) — #965 font bake; overlaps with #243's font fix.
4. The remaining single-commit "salvage WIP" tips are mostly checkpoint commits;
   review individually to confirm nothing load-bearing was left behind.

## Recommendation

1. Merge `reconcile3/converge-20260531` → `main` as the main consolidation (this PR).
2. Then bring PR #243 (`fix/modern-loading-bar-20260531`) on top — iter-153 session work.
3. Triage the unique-tail branches above (especially `fix/assetswap-runtime-bugs-20260530`)
   and PR/cherry-pick anything not already captured.
4. Keep all branches intact (no deletions) until consolidation is merged and verified.
