# ADR: FR-LOD-001 -- Level of detail

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-LOD-001
> Epic: FR-LOD

## Context

FR-LOD-001 is part of the FR-LOD epic. This functional requirement captures: Level of detail.

Implementing crate: `crates/engine/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/engine/src/lod.rs:94`
- `crates/engine/tests/fr_matrix_batch1.rs:13`
- `crates/engine/tests/fr_matrix_batch1.rs:44`
- `crates/engine/tests/fr_matrix_batch1.rs:45`
- `crates/engine/tests/fr_matrix_batch1.rs:46`
- `crates/engine/tests/fr_matrix_batch3.rs:18`
- `crates/engine/tests/fr_matrix_batch3.rs:19`

## Decision

TBD -- The architectural decision for FR-LOD-001 needs to be finalized based on implementation exploration.

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
