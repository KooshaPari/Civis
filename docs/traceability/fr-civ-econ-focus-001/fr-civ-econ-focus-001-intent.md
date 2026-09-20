# Intent: FR-CIV-ECON-FOCUS-001 -- Per-settlement economic focus (durable)

> Date: 2026-09-19
> FR: FR-CIV-ECON-FOCUS-001
> Epic: FR-CIV-ECON-FOCUS

## User Intent

The product owner requires Per-settlement economic focus (durable) as part of the FR-CIV-ECON-FOCUS epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Per-settlement economic focus (durable) is properly specified, implemented, and testable within the simulation engine.

`WorldState::econ_focus: BTreeMap<u32, EconomicFocus>` is a
Simulation-owned field that records the resource-allocation policy
each settlement has chosen (Agrarian, Industrial, Sacred, Mercantile).
It is mutated every tick by `phase_policy_econ` and mirrored into the
save bundle via `save_state_mirror` / `save_state_mirror_to`, so
settlement policy choices survive archive round-trip exactly.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS.
FR-CIV-ECON-FOCUS-001 contributes to the economic policy loop by
keeping the chosen focus durable; if it were lost on reload, every
settlement would reset to default policy and lose its accumulated
production bias.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/engine/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Persistence round-trip exact
      (`crates/engine/tests/institutions_buildsites_econfocus_persistence.rs`)

### How We Know This FR Is Satisfied

1. `cargo test -p civ-engine institutions_buildsites_econfocus` passes
2. Two `Simulation::with_seed(SEED)` instances produce byte-identical
   `econ_focus` maps
3. Save/load round-trip preserves the exact per-settlement focus value
4. The same entry is exercised by
   `crates/engine/tests/persistence_replay_coverage.rs`

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-econ-focus-001-intent.md` |
| Source | `crates/engine/src/engine.rs:472` (field), `:2183` (mirror) |
| Tests | `crates/engine/tests/institutions_buildsites_econfocus_persistence.rs:4`, `crates/engine/tests/persistence_replay_coverage.rs:14` |

<!-- Covers: FR-CIV-ECON-FOCUS-001 -->
