# ADR: FR-ECON-001 -- Economics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-ECON-001
> Epic: FR-ECON

## Context

FR-ECON-001 is part of the FR-ECON epic. This functional requirement captures: Economics.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `crates/economy/src/lib.rs:106`
- `crates/engine/src/engine.rs:2011`
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:260`

### Test Coverage
- `crates/economy/src/lib.rs:308`
- `crates/economy/tests/fr_matrix_batch7.rs:9`
- `crates/economy/tests/fr_matrix_batch7.rs:135`
- `crates/economy/tests/fr_matrix_batch7.rs:138`
- `crates/economy/tests/fr_matrix_batch7.rs:154`
- `crates/economy/tests/fr_matrix_batch7.rs:165`

## Decision

TBD -- The architectural decision for FR-ECON-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Economics requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
