# ADR: FR-API-001 -- API

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-API-001
> Epic: FR-API

## Context

FR-API-001 is part of the FR-API epic. This functional requirement captures: API.

Implementing crate: `crates/server/src/`

### Referenced Source
- `crates/engine/src/scenario.rs:1`
- `docs/guides/scenario-yaml.md:3`
- `docs/IMPLEMENTATION_STATUS.md:33`
- `docs/reference/agileplus-artifacts-index.md:239`
- `docs/reference/agileplus-artifacts-index.md:307`

### Test Coverage
- `crates/engine/src/scenario.rs:274`
- `crates/engine/tests/fr_matrix_batch1.rs:18`
- `crates/engine/tests/fr_matrix_batch1.rs:351`
- `crates/engine/tests/fr_matrix_batch1.rs:352`
- `crates/engine/tests/fr_matrix_batch1.rs:353`
- `crates/engine/tests/fr_matrix_batch1.rs:382`
- `crates/engine/tests/fr_matrix_batch1.rs:383`
- `crates/engine/tests/fr_matrix_batch3.rs:161`

## Decision

TBD -- The architectural decision for FR-API-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/server/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the API requirement in the simulation

### Negative
- Adds complexity to the server crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/server/`
2. **Option B**: Extract into a dedicated sub-crate
