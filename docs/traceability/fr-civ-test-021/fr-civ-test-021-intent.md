# Intent: FR-CIV-TEST-021 -- civ-server save/load end-to-end

> Date: 2026-09-19
> FR: FR-CIV-TEST-021
> Epic: FR-CIV-TEST

## User Intent

The product owner requires civ-server save/load end-to-end as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the end-to-end save/load round-trip
test for `civ-server`:

- `save_load_round_trip_preserves_world_state` — five-tick sim with a
  world-state fixture saves to disk, loads, and yields identical (tick,
  population, belief, cohesion, unrest).
- `post_load_ticks_are_deterministic` — two independent loads + post-load
  ticks produce identical snapshots.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-021 contributes to the overall simulation capability by
addressing: external validation that the `CivSaveBundle` archive format is
stable across save and load, and that post-load ticks are deterministic.

## Acceptance Signal

### Definition of Done

- [x] `crates/server/tests/save_load_e2e.rs` exists with the two e2e tests.
- [x] `cargo test -p civ-server --test save_load_e2e` passes.

### How We Know This FR Is Satisfied

1. Saving and loading preserves the world-state fixture exactly.
2. Two independent loads + post-load ticks yield identical snapshots.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/server/tests/save_load_e2e.rs` |
| Implementing crate | `crates/server/` |
