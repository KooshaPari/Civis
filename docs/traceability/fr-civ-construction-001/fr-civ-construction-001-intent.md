# Intent: FR-CIV-CONSTRUCTION-001 -- Construction-site persistence

> Date: 2026-09-19
> FR: FR-CIV-CONSTRUCTION-001
> Epic: FR-CIV-CONSTRUCTION

## User Intent

The product owner requires Construction-site persistence as part of the FR-CIV-CONSTRUCTION epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that every per-settlement build site
(`civ_build::BuildSite`) is durable across archive (save/load) round-trips and
that the `phase_construction_sites` tick-phase has authoritative ownership of
the live `Simulation::build_sites` vector.

Specifically it:

- Adds `build_sites: Vec<civ_build::BuildSite>` to `WorldState` (defaulted for
  legacy v3 archive compatibility).
- Mirrors `Simulation::build_sites` into `WorldState::build_sites` immediately
  before serialization so a fresh archive restores the exact set of in-progress
  constructions.
- Gates the persistence round-trip behind the existing
  `institutions_buildsites_econfocus_persistence.rs` and
  `persistence_replay_coverage.rs` integration tests.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-CONSTRUCTION-001 contributes to the overall simulation capability by
addressing: persistence of mid-build structures so a reloaded archive doesn't
silently reset in-progress construction progress for any settlement.

## Acceptance Signal

### Definition of Done

- [x] `WorldState::build_sites` field exists with `#[serde(default)]` and is mirrored in `state.build_sites = self.build_sites.clone()` before serialization.
- [x] `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs` covers the round-trip.
- [x] `crates/engine/tests/persistence_replay_coverage.rs` covers the replay path.
- [x] `cargo build -p engine` passes.
- [x] No regressions in existing FRs.

### How We Know This FR Is Satisfied

1. `cargo test -p engine institutions_buildsites_econfocus_persistence` passes.
2. `cargo test -p engine persistence_replay_coverage` passes.
3. A saved archive with N in-progress construction sites reloads to exactly N sites.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/engine.rs::build_sites` field + state mirror |
| Tests | `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs` |
| Tests | `crates/engine/tests/persistence_replay_coverage.rs` |
| Implementing crate | `crates/engine/` |
