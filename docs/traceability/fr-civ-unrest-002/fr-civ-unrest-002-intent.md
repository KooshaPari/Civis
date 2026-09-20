# Intent: FR-CIV-UNREST-002 -- Per-settlement riot + migrant accumulators (durable)

> Date: 2026-09-19
> FR: FR-CIV-UNREST-002
> Epic: FR-CIV-UNREST

## User Intent

The product owner requires Per-settlement riot + migrant accumulators (durable) as part of the FR-CIV-UNREST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Per-settlement riot + migrant accumulators (durable) is properly specified, implemented, and testable within the simulation engine.

`WorldState::riot_accumulator` and `WorldState::migrant_accumulator` are
two Simulation-owned `BTreeMap<u32, i64>` fields. The riot accumulator
tracks per-settlement unrest pressure that, once it crosses a
threshold, triggers a riot event. The migrant accumulator tracks the
net flow of migrants per settlement (positive = inflow, negative =
outflow). Both are mutated every tick by `phase_unrest` and mirrored
into the save bundle via `save_state_mirror` / `save_state_mirror_to`
so unrest state survives archive round-trip exactly.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-UNREST-002 contributes to the unrest / migration loop by
keeping the per-settlement counters durable. Without this, every save
load would reset the unrest trajectory and the simulation would lose
its mid-crisis pressure. Legacy v3 saves (no field) deserialize to
empty maps.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Persistence round-trip exact for both maps (positive, negative, zero)
- [x] Same-seed determinism asserted

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine riot_migrant_taxation` passes
2. Two `Simulation::with_seed(SEED)` instances share the same accumulator maps
3. Save/load round-trip preserves positive, negative, and zero values
4. Empty accumulators + default `scenario_taxation` survive reload
   (backward-compat with v3 saves)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-unrest-002-intent.md` |
| Source | `crates/engine/src/engine.rs:504`, `:526`, `:2200` |
| Tests | `crates/engine/tests/riot_migrant_taxation_persistence.rs:2` |

<!-- Covers: FR-CIV-UNREST-002 -->
