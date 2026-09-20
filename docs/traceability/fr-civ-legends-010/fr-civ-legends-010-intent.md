# Intent: FR-CIV-LEGENDS-010 -- Legend significance accumulator

> Date: 2026-09-19
> FR: FR-CIV-LEGENDS-010
> Epic: FR-CIV-LEGENDS

## User Intent

The product owner requires Legend significance accumulator as part of the FR-CIV-LEGENDS epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the live `Simulation`-owned
`SignificanceAccumulator` (`civ_legends::significance::SignificanceAccumulator`)
that the legends subsystem uses to track per-entity historical significance
across the run. The mirror lives on `WorldState` (FR-CIV-LEGENDS-001) so
archive round-trips preserve significance state.

Specifically:

- `crates/engine/src/engine.rs::significance` field on `Simulation`
  (FR-CIV-LEGENDS-010) — accumulator mutated by the legends subsystem.
- Companion field on `WorldState` for archive persistence.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-LEGENDS-010 contributes to the overall simulation capability by
addressing: the live accumulator that drives legend generation. Persistence
mirror lives under FR-CIV-LEGENDS-001.

## Acceptance Signal

### Definition of Done

- [x] `Simulation::significance` field exists and is wired into the legends subsystem.
- [x] Companion field on `WorldState` ensures persistence (FR-CIV-LEGENDS-001).
- [x] `cargo build -p engine` passes.

### How We Know This FR Is Satisfied

1. The legends subsystem compiles and updates the accumulator.
2. Save/load preserves significance via the WorldState mirror.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/engine.rs::significance` |
| Implementing crate | `crates/engine/` |
