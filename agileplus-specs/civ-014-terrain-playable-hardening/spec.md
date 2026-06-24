---
spec_id: civ-014
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-09
---

# Specification: Terrain Playability Hardening

**Slug**: civ-014-terrain-playable-hardening | **Epic**: E7 | **Date**: 2026-06-09 | **State**: ACTIVE

## Problem Statement

The wave-1 / wave-1-emergence worktrees introduced several in-flight terrain fixes
that have no spec home: terrain fragmentation / chunk-seam, CA-dirty-chunk
performance, map2d zoom, map2d UX (#2494), water-placement, actor-y fix, and
emergence-spawn layout. These all share a single property — they are **playability
gates** that block a user from running an end-to-end demo on a freshly-built
civ-watch / civ-server pair. Without a spec, the gate criteria drift between
agents; with this spec, every terrain playability fix has a uniform contract:
"the watcher surface stays in a playable state from cold boot to first actor
spawn, deterministically, with documented performance budgets."

## Target Users

- Bevy primary / Godot secondary / Unreal show client engineers shipping terrain
  fixes
- QA / agent-smoke authors extending `scripts/agent-smoke.ps1`
- Engine / voxel crate maintainers reviewing dirty-chunk / seam interactions

## Functional Requirements

- [ ] **FR-CIV-TERRAIN-001**: A terrain playability smoke SHALL be runnable from cold
  boot (`cargo run -p civ-watch` + `cargo run -p civ-server` + a client attach)
  and reach a state where a user can pan, zoom, paint a voxel, and observe the
  paint reflected in the watch snapshot within 10 s wall-clock on reference
  hardware. (`agent-smoke.ps1` may satisfy this; AC ties to that gate.)
- [ ] **FR-CIV-TERRAIN-002**: `civ-voxel` chunk seams SHALL be free of visible
  artifacts at all zoom levels supported by the 2D map and 3D clients
  (no z-fighting, no missing cells at the 16³ leaf boundary).
- [ ] **FR-CIV-TERRAIN-003**: The CA-dirty-chunk path SHALL keep the P99 frame
  delta below 16 ms for a 64×64 chunk grid with random writes at 1 % density
  (measured by `bench_ca_dirty_chunk`).
- [ ] **FR-CIV-TERRAIN-004**: Map2D zoom levels SHALL round-trip without voxel-data
  loss; `map2d.zoom` and `map2d.ux` (issue #2494) SHALL be expressed in the
  watch HTTP surface and the web dashboard.
- [ ] **FR-CIV-TERRAIN-005**: Water placement tools SHALL respect a single source of
  truth (the `civ-watch` water endpoint / `civ-voxel` material id) across all
  three reference clients; no client-side hard-coded water ids.
- [ ] **FR-CIV-TERRAIN-006**: Emergent actor Y-axis (height) fix SHALL persist
  deterministically across save/load and replay; an actor spawned at height H
  in tick T SHALL appear at height H in any later snapshot of the same run.

## Non-Functional Requirements

- Crates: `civ-voxel`, `civ-watch`, `civ-server`, `clients/{bevy-ref,godot-ref,unreal-show}`
- Determinism: every playability smoke must yield identical first-frame hashes
  for a given seed (binds to FR-CORE-002 / FR-CIV-VOXEL-002)
- The "playable" gate is **a single command**, not a checklist:
  `just civis-3d-verify` (catalog + scenario + web + mod-host + check/clippy/fmt)
  plus `.\scripts\agent-smoke.ps1` for the WS + watch + Unreal preflight path.
- Per-client timeout budget documented in `docs/guides/client-attach-matrix.md`.

## Constraints and Dependencies

- Depends on FR-CIV-VOXEL-001/002/003 (adaptive storage, dirty queue, fixed-point)
- Depends on FR-PROTO-001/002 (WebSocket + JSON-RPC) for the end-to-end smoke
- Depends on FR-CIV-UX-006 (spawn palette) for the actor-spawn half of the smoke
- Conflicts-of-record: `wt/actor-y-fix`, `wt/chunk-seam`, `perf/ca-dirty-chunk`,
  `wt/map-seed`, `wt/map2d-zoom`, `wt/map2d-ux-2494`, `wt/water-placement`,
  `wt/emergence-spawn` — these branches are the land-surface for this spec's
  AC tests.

## Acceptance Criteria

- [ ] `.\scripts\agent-smoke.ps1` exits 0 with the playability checks enabled
- [ ] `just civis-3d-verify` exits 0 with no `cargo clippy --all-targets -- -D warnings` regressions
- [ ] `bench_ca_dirty_chunk` records P99 < 16 ms in CI artifact
- [ ] Map2D zoom round-trip test (`map2d_zoom_round_trip`) passes
- [ ] Chunk-seam visual-diff test (`chunk_seam_watertight`) passes
- [ ] Actor-Y determinism test (`actor_y_persists_across_replay`) passes

## Implementation Notes

- The wave-1 PR stack (PR #340 → follow-up #343 → #344) provides the spec
  triple layout (`agileplus-specs/civ-XXX/<slug>/{spec.md,plan.md,meta.json}`).
- The terrain playability gate lives in `scripts/agent-smoke.ps1` (the AX-04
  entry in `docs/development-guide/fr-ax-dx-ux-maturity-audit.md`).
- Existing branches: keep names as `wt/<topic>` for new fixes; use `fix/<topic>`
  only when the change has an associated GitHub issue link.

## Status

| Story | Status |
|-------|--------|
| E7.1 Terrain playability smoke | Partial (scripted in `agent-smoke.ps1`; spec home is this file) |
| E7.2 Chunk-seam fix (`wt/chunk-seam`) | Partial |
| E7.3 CA-dirty-chunk perf (`perf/ca-dirty-chunk`) | Partial |
| E7.4 Map2D zoom + UX (`wt/map2d-zoom`, `wt/map2d-ux-2494`) | Partial |
| E7.5 Water placement single-source-of-truth (`wt/water-placement`) | Partial |
| E7.6 Actor-Y determinism (`wt/actor-y-fix`) | Partial |
| E7.7 Emergence spawn layout (`wt/emergence-spawn`) | Partial |
| E7.8 Map seed determinism (`wt/map-seed`) | Partial |
