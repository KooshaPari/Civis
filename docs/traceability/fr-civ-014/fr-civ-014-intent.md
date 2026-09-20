# Intent: FR-CIV-014 -- Determinism for 3D position round-trips

> Date: 2026-09-19
> FR: FR-CIV-014
> Epic: FR-CIV

## User Intent

The product owner requires Determinism for 3D position round-trips as part of the FR-CIV epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that the simulation produces identical, reproducible
state when (a) world snapshots and replay files are serialised + reloaded, and (b) seed
selection chooses the same `SeedDefinition` regardless of insertion order in `SeedLibrary`.

Specifically it locks in:

- `Position3d` (x, y, z) survives both save/load (`crates/engine/src/save.rs`) and replay
  round-trip (`crates/engine/src/save.rs::actor_y_persists_across_replay`).
- `select_seed_for_position` deterministically picks the biome-matching seed in
  `SeedLibrary` even when multiple candidates are present
  (`crates/engine/src/emergence.rs::seed_selection_is_deterministic_across_library_ordering`).

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-014 contributes to the overall simulation capability by addressing: determinism
guarantees for 3D-coord-aware persistence and seed ordering, which underpins replay
correctness in `civ-server` and snapshot diffs in `civ-watch`.

## Acceptance Signal

### Definition of Done

- [x] `cargo test -p engine --lib save` passes `save_and_load_preserves_position3d_y`.
- [x] `cargo test -p engine --lib save` passes `actor_y_persists_across_replay`.
- [x] `cargo test -p engine --lib emergence` passes `seed_selection_is_deterministic_across_library_ordering`.
- [ ] Tag source with `// FR-CIV-014` near the defining functions/tests (added during P3 audit).
- [x] No regressions in existing FRs (verified by P3 build).

### How We Know This FR Is Satisfied

1. `cargo test -p engine save::` passes both `position3d_y` round-trip tests.
2. `cargo test -p engine emergence::seed_selection_is_deterministic` passes.
3. The simulation runs without errors related to Determinism for 3D position round-trips.
4. The feature is observable in the simulation output (replay and load yield identical positions).

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/engine/src/emergence.rs::seed_selection_is_deterministic_across_library_ordering` |
| Tests | `crates/engine/src/save.rs::save_and_load_preserves_position3d_y` |
| Tests | `crates/engine/src/save.rs::actor_y_persists_across_replay` |
| Implementing crate | `crates/engine/` |
