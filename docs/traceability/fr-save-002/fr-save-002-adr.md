# ADR: FR-SAVE-002 -- Save system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SAVE-002
> Epic: FR-SAVE

## Context

FR-SAVE-002 is part of the FR-SAVE epic. This functional requirement captures: Save system.

Implementing crate: `crates/save-db/src/`

### Referenced Source
- `crates/engine/src/replay.rs:297`
- `docs/specs/CIV-1000-save-load-persistence-spec.md:2801`

### Test Coverage
- `crates/engine/src/save.rs:278`
- `crates/engine/tests/fr_matrix_batch1.rs:17`
- `crates/engine/tests/fr_matrix_batch1.rs:322`
- `crates/engine/tests/fr_matrix_batch1.rs:323`
- `crates/engine/tests/fr_matrix_batch1.rs:324`
- `crates/engine/tests/fr_matrix_batch3.rs:151`
- `crates/engine/tests/fr_matrix_batch3.rs:152`

## Decision

TBD -- The architectural decision for FR-SAVE-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/save-db/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Save system requirement in the simulation

### Negative
- Adds complexity to the save-db crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/save-db/`
2. **Option B**: Extract into a dedicated sub-crate
