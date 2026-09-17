# ADR: FR-LOD-002 -- Level of detail

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-LOD-002
> Epic: FR-LOD

## Context

FR-LOD-002 is part of the FR-LOD epic. This functional requirement captures: Level of detail.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/engine/src/lod.rs:63`

### Test Coverage
- `crates/engine/src/lod.rs:102`
- `crates/engine/tests/fr_matrix_batch1.rs:13`
- `crates/engine/tests/fr_matrix_batch1.rs:60`
- `crates/engine/tests/fr_matrix_batch1.rs:61`
- `crates/engine/tests/fr_matrix_batch1.rs:62`
- `crates/engine/tests/fr_matrix_batch3.rs:27`
- `crates/engine/tests/fr_matrix_batch3.rs:28`

## Decision

TBD -- The architectural decision for FR-LOD-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Level of detail requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
