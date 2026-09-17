# ADR: FR-CORE-005 -- Core system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CORE-005
> Epic: FR-CORE

## Context

FR-CORE-005 is part of the FR-CORE epic. This functional requirement captures: Core system.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/engine/src/hash_chain.rs:1`
- `docs/reference/agileplus-artifacts-index.md:41`
- `docs/reference/agileplus-artifacts-index.md:257`

### Test Coverage
- `crates/engine/src/hash_chain.rs:212`
- `crates/engine/tests/fr_matrix_batch1.rs:14`
- `crates/engine/tests/fr_matrix_batch1.rs:126`
- `crates/engine/tests/fr_matrix_batch1.rs:127`
- `crates/engine/tests/fr_matrix_batch1.rs:128`
- `crates/engine/tests/fr_matrix_batch3.rs:54`
- `crates/engine/tests/fr_matrix_batch3.rs:55`

## Decision

TBD -- The architectural decision for FR-CORE-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core system requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
