# Intent: FR-CIV-AGGRESSION-001 -- Per-faction aggression sentiment

> Date: 2026-09-19
> FR: FR-CIV-AGGRESSION-001
> Epic: FR-CIV-AGGRESSION

## User Intent

The product owner requires per-faction aggression sentiment as part of the FR-CIV-AGGRESSION epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that per-faction aggression sentiment is properly specified, implemented, and testable within the simulation engine.

`WorldState::faction_aggression: BTreeMap<u32, f32>` is a Simulation-owned
field that stores a per-faction aggression scalar in `[0.0, 1.0]`. It is
mutated every tick by `phase_aggression` and mirrored into the save
bundle by `save_state_mirror` / `save_state_mirror_to` so archive
round-trips preserve the exact sentiment value.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-AGGRESSION-001 contributes to the diplomacy / AI subsystem by
exposing a single per-faction pressure that downstream aggression,
war-trigger, and treaty-cooling phases can read.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Persistence round-trip exact (`crates/engine/tests/culture_ideology_aggression_persistence.rs`)
- [x] Same-seed determinism asserted

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine culture_ideology_aggression` passes
2. Two `Simulation::with_seed(SEED)` instances produce byte-identical
   `faction_aggression` maps
3. Save/load round-trip preserves the exact per-faction values

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-aggression-001-intent.md` |
| Source | `crates/engine/src/engine.rs:492` (field), `:2191` (mirror) |
| Tests | `crates/engine/tests/culture_ideology_aggression_persistence.rs:3` |

<!-- Covers: FR-CIV-AGGRESSION-001 -->
