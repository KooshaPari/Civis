# L4 Branch Triage — 12 preserved local feature branches (2026-06-13)

All 12 worktrees removed; branches preserved as local refs. Cherry-equivalence check (`git cherry HEAD <br>`) shows **all 12 are cherry-unique vs HEAD** — none already content-merged. Each carries un-landed work.

| Branch | ahead | files | nature | plan |
|---|---|---|---|---|
| feat/sw-building-bundles-20260531 | 1 | 163 | URP-shader unit-bundle rebuild | **MERGE** (substantial, real assets) |
| feat/rigging-optimizer-20260531 | 3 | 5 | blender rig+decimate scaffold (#991) | **MERGE** (real feature) |
| feat/uicensus-20260601 | 1 | 3 | game_dump_ui_tree alias + ui-census tooling | MERGE |
| fix/sw-tmp-font-20260531 | 2 | 3 | import TMP essentials before headless build | MERGE |
| feat/loadingscreen-20260531 | 2 | 4 | native-symbol gap docs | MERGE |
| feat/modern-20260531 | 1 | 11 | wip salvage (modern reskin) | MERGE (review 11 files) |
| feat/cursors-20260601 | 1 | 6 | wip salvage (cursors) | MERGE |
| feat/icons-20260601 | 1 | 5 | wip salvage (icons) | MERGE |
| feat/bldicons-20260531 | 1 | 4 | wip salvage (build icons) | MERGE |
| fix/cursor-agent-config-resolution-20260531 | 1 | 2 | wip salvage (cursor cfg) | MERGE |
| rnd/brickalyzer-20260531 | 1 | 2 | brick-mode feasibility docs | MERGE |
| feat/naval-20260531 | 1 | 1 | wip salvage (naval) | MERGE |

## Consolidation strategy (user: "consolidate to 1 branch BEFORE deeper eval")
Rather than 12 separate PRs (12 × 2.5min lefthook gates), **merge all 12 branch tips onto the integration branch (HEAD = wsm/agileplus-dag-20260610) locally**, resolving conflicts, gate-green once, then ONE push + ONE PR lands everything → main.

Order: smallest/safest first (naval, brickalyzer, cursor-cfg, bldicons), then tooling (uicensus, sw-tmp-font, loadingscreen), then bigger (icons, cursors, modern, rigging-optimizer), then the 163-file sw-building-bundles last. Build-gate after the bundle merge.

Caveat: many are "salvage pre-capacity-death" wip snapshots from 2026-05-31 — content may overlap work already on main via OTHER paths (cherry says commit-unique, but file content may be superseded). Per-merge, prefer `-X theirs`/`-X ours` only after inspecting; default to manual conflict resolution. Drop a branch if its merge is a pure no-op diff.

## Already done
- 12 already-merged REMOTE branches deleted (remote heads 41→22).
- Cleanup commit 1a552447 + L1 (99 dirty, CI-green) landed on remote (HEAD 76099bf7 verified).

## L4 MERGE RESULTS (2026-06-13)
**8/12 merged CLEAN onto HEAD** (no-ff): naval, brickalyzer, cursor-cfg, bldicons, uicensus, sw-tmp-font, loadingscreen, modern. HEAD → 9672fe75. Gate running.

**3 CONFLICTED** (aborted, need manual resolution):
- feat/icons-20260601 → `src/Runtime/UI/MainMenuThemer.cs` (content)
- feat/cursors-20260601 → `src/Runtime/Plugin.cs` (content)
- feat/rigging-optimizer-20260531 → `docs/sessions/rigging-optimizer-pipeline-20260531.md` + `packs/warfare-starwars/assets/tools/blender_rig_and_decimate.py` (add/add)

**1 NOT YET ATTEMPTED**: feat/sw-building-bundles-20260531 (163 files — last, after conflicts resolved).

Next: gate+push the 8 clean → then resolve 3 conflicts individually (Plugin.cs/MainMenuThemer.cs are core — careful manual merge, prefer HEAD's structure + graft branch's unique additions) → then sw-building-bundles → ONE PR to main → git branch -d all 12.
