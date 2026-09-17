# ADR: FR-MOD-004 -- Modding

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-MOD-004
> Epic: FR-MOD

## Context

FR-MOD-004 is part of the FR-MOD epic. This functional requirement captures: Modding.

Implementing crate: `crates/mod-host/src/`

### Referenced Source
- `crates/engine/src/engine.rs:1227`
- `crates/engine/src/replay.rs:51`
- `crates/engine/src/replay.rs:63`
- `crates/engine/src/replay.rs:82`
- `crates/engine/src/replay.rs:273`
- `crates/engine/src/replay.rs:285`
- `crates/mod-host/src/lib.rs:204`
- `crates/mod-host/src/lib.rs:217`

### Test Coverage
- `crates/engine/src/scenario.rs:347`
- `crates/engine/tests/fr_matrix_batch1.rs:16`
- `crates/engine/tests/fr_matrix_batch1.rs:263`
- `crates/engine/tests/fr_matrix_batch1.rs:264`
- `crates/engine/tests/fr_matrix_batch1.rs:265`
- `crates/engine/tests/fr_matrix_batch1.rs:303`
- `crates/engine/tests/fr_matrix_batch3.rs:120`
- `crates/engine/tests/fr_matrix_batch3.rs:121`

## Decision

TBD -- The architectural decision for FR-MOD-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/mod-host/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Modding requirement in the simulation

### Negative
- Adds complexity to the mod-host crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/mod-host/`
2. **Option B**: Extract into a dedicated sub-crate
