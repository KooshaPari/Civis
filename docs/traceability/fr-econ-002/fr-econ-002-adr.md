# ADR: FR-ECON-002 -- Economics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-ECON-002
> Epic: FR-ECON

## Context

FR-ECON-002 is part of the FR-ECON epic. This functional requirement captures: Economics.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:261`

### Test Coverage
- `crates/economy/src/allocator.rs:765`
- `crates/economy/src/allocator.rs:816`
- `crates/economy/src/lib.rs:381`
- `crates/economy/tests/fr_matrix_batch7.rs:10`
- `crates/economy/tests/fr_matrix_batch7.rs:180`
- `crates/economy/tests/fr_matrix_batch7.rs:183`
- `crates/economy/tests/fr_matrix_batch7.rs:223`

## Decision

TBD -- The architectural decision for FR-ECON-002 needs to be finalized based on implementation exploration.

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
