# Intent: FR-CIV-ECON-010 -- Scenario taxation policy

> Date: 2026-09-19
> FR: FR-CIV-ECON-010
> Epic: FR-CIV-ECON

## User Intent

The product owner requires Scenario taxation policy as part of the FR-CIV-ECON epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines a `civ_economy::Taxation` policy that is
applied at the scenario level (rather than per-settlement) and persists across
archive (save/load) round-trips.

Specifically it:

- Adds `scenario_taxation: civ_economy::Taxation` to both `WorldState` (durable)
  and `Simulation` (live, mutated by `apply_scenario_taxation`).
- Mirrors `Simulation::scenario_taxation` into `WorldState::scenario_taxation`
  before serialization so a saved archive preserves the scenario's tax policy.
- Is exercised by `crates/engine/tests/riot_migrant_taxation_persistence.rs`.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-ECON-010 contributes to the overall simulation capability by addressing:
scenario-level taxation policy that drives the per-settlement economy and the
riot / migrant accumulator coupling (FR-CIV-UNREST-002).

## Acceptance Signal

### Definition of Done

- [x] `WorldState::scenario_taxation` field exists with `#[serde(default)]`.
- [x] `Simulation::scenario_taxation` field exists and is mirrored into `WorldState` pre-serialize.
- [x] `crates/engine/tests/riot_migrant_taxation_persistence.rs` covers the round-trip.
- [x] `cargo build -p engine` passes.

### How We Know This FR Is Satisfied

1. `cargo test -p engine riot_migrant_taxation_persistence` passes.
2. A saved archive restores scenario-level taxation exactly.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/engine.rs::scenario_taxation` field + state mirror |
| Tests | `crates/engine/tests/riot_migrant_taxation_persistence.rs` |
| Implementing crate | `crates/engine/` |
