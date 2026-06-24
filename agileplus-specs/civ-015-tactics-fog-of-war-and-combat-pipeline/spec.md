---
spec_id: civ-015
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-09
---

# Specification: Tactics, Fog-of-War & Combat Pipeline (PR #309/310)

**Slug**: civ-015-tactics-fog-of-war-and-combat-pipeline | **Epic**: E4 | **Date**: 2026-06-09 | **State**: ACTIVE

## Problem Statement

PR #309 (`feat(tactics): fog of war + obstacle A* + replay determinism`),
PR #310 (`feat(dashboard): tactics panel, fog of war overlay, unit selection`),
and the broader FR-CIV-TACTICS-042+ workstream ship the engagement-side combat
pipeline: line-of-sight, formations, war bridge, A* obstacle routing, occupied-
cell blocking, fog-of-war gating, combat replay hash chain, and the
dashboard-side tactics panel + fog overlay. These land in `civ-tactics`,
`crates/engine/src/{engine,replay,hash_chain}.rs`, `crates/watch`, the
JSON-RPC snapshot surface, and `web/dashboard`. There is no single
authoritative spec for the **stack** even though `fr-3d-matrix.md` tracks
individual rows. This spec is the home for the cross-cutting pipeline and
the fog-of-war contract.

## Target Users

- Tactics crate developers extending `civ-tactics` (operational layer, war
  bridge, fog, doctrine)
- Engine / replay engineers wiring combat into the hash chain
- Web dashboard authors adding the tactics panel and fog overlay
- QA / replay-determinism CI authors

## Functional Requirements

- [ ] **FR-CIV-TACTICS-100**: The `civ-tactics` crate SHALL expose a deterministic
  pipeline: LOS → formation → war bridge (with cadence) → fog gate → combat
  resolution → `DamageEvent` → replay log → hash chain. Each stage has a
  unit test asserting stage-local determinism for a fixed seed.
- [ ] **FR-CIV-TACTICS-101**: A* obstacle pathfinding SHALL refuse to step
  through a `grid_cell_blocked` cell and SHALL prefer the lexicographically
  smallest valid path under equal cost (stable iteration order).
- [ ] **FR-CIV-TACTICS-102**: Combat engagements SHALL be recorded in
  `ReplayEvent::Combat` and SHALL extend the replay hash chain
  (`FR-CIV-TACTICS-041`); combat-replay determinism is the gate
  (`replay_combat_log_deterministic_for_seed_rerun`).
- [ ] **FR-CIV-FOG-001**: Fog-of-war visibility SHALL be a function of `(unit
  position, vision radius, terrain LOS)`. A target is "visible" iff the
  line-of-sight check from any friendly unit passes; vision state is
  deterministic given the same unit set and terrain.
- [ ] **FR-CIV-FOG-002**: The war bridge SHALL NOT queue `DamageEvent`s for
  engagements where the attacker has no friendly unit with LOS to the
  defender (fog gating, FR-CIV-TACTICS-042).
- [ ] **FR-CIV-FOG-003**: Scenario `fog_vision_radius` SHALL be respected
  per-civilisation; the default baseline scenario SHALL set it to 4 hex
  for at least one side (FR-CIV-TACTICS-045).
- [ ] **FR-CIV-FOG-004**: The web dashboard SHALL provide a tactics panel
  with unit selection, an at-a-glance fog overlay, and a jump-to-engagement
  action; the panel reads from `sim.snapshot` JSON-RPC only — no private
  state channel (FR-CIV-TACTICS-054).
- [ ] **FR-CIV-FOG-005**: Fog-of-war observer mode (per the legacy
  FR-SESSION-014) SHALL apply server-side visibility filtering before
  transmission; the observer SHALL never receive hidden state. The
  per-nation filter MUST be stable for a given (snapshot, fog_nation_id).

## Non-Functional Requirements

- Determinism: every combat and fog decision is a pure function of
  `(tick, seed, scenario, snapshot)`; no wall-clock in any hot path
- Test surface: `crates/tactics/`, `crates/engine/src/{engine,replay,
  hash_chain}.rs`, `crates/watch`, `server/jsonrpc`, `web/dashboard`
- The `ws_jsonrpc_sim_snapshot_returns_snapshot_fields` integration
  test is the gate for dashboard smoke (asserts `civ_pins[].job`)
- Performance: the fog gate is O(units × grid) per military cadence; the
  grid must be cached across ticks for unchanged terrain

## Constraints and Dependencies

- Depends on FR-CIV-VOXEL-001/002 (voxel substrate + dirty queue)
- Depends on FR-CIV-AGENTS-001 (wardrobe/tools per agent)
- Depends on FR-PROTO-002 (JSON-RPC) for the dashboard read path
- Depends on FR-REPLAY-001 (`.civreplay` format) for the combat log
- Refines FR-CIV-TACTICS-000 through -077 (the matrix rows in
  `docs/traceability/fr-3d-matrix.md`); this spec is the *pipeline-level*
  umbrella and does not duplicate the per-row tests

## Acceptance Criteria

- [ ] `cargo test -p civ-tactics` exits 0; `los`, `formation`, `war_bridge`,
  `fog`, `astar_path`, `pathfinding_bfs_steps_toward_enemy`,
  `operational_movement_*` all pass
- [ ] `cargo test -p civ-engine pending_damage war_bridge_records
  combat_events_extend_replay_hash_chain
  replay_combat_log_deterministic_for_seed_rerun
  replay_combat_events_restore_pending_damage
  replay_combat_drains_to_same_voxel_state_as_live` all pass
- [ ] `cargo test -p civ-tactics fog_blocks_engagement_beyond_vision
  scenario_fog_wires_military_phase scenario_military_wires_military_phase`
  pass
- [ ] `node --test web/dashboard` (or `npm test` in `web/`) covers
  `fog_overlay` and `tactics_panel` component smoke
- [ ] `.\scripts\agent-smoke.ps1` is green end-to-end (determinism gate)

## Implementation Notes

- Existing `docs/development-guide/p-w1-kickoff.md` enumerates the
  per-row FR status; treat that as the per-FR check-list. This spec
  is the *pipeline* and *fog contract* authoritative home.
- PR #309 is the canonical merge for the pipeline; PR #310 is the
  dashboard surface; both are ref-only against the same `civ-tactics`
  engine state.
- Fog of war in `crates/tactics/fog_of_war.rs` is the single source of
  truth — client overlays and the observer filter both read it (server
  applies it pre-snapshot).

## Status

| Story | Status |
|-------|--------|
| E4.1 LOS / formation / war bridge | implemented (rows 020–022) |
| E4.2 A* obstacle routing + occupied-cell | implemented (rows 036–037, 039) |
| E4.3 Fog-of-war gating in war bridge | implemented (row 042) |
| E4.4 Replay combat + hash chain | implemented (rows 025, 041) |
| E4.5 Scenario fog fields | implemented (rows 045, 050) |
| E4.6 Dashboard tactics panel + fog overlay (PR #310) | implemented |
| E4.7 Server-side fog observer filter (FR-SESSION-014) | implemented in CIV-0900 |
