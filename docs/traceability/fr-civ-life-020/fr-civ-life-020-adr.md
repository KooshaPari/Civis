# ADR: FR-CIV-LIFE-020 -- Life simulation and needs

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LIFE-020
> Epic: FR-CIV-LIFE

## Context

FR-CIV-LIFE-020 is part of the FR-CIV-LIFE epic. This functional requirement captures: Life simulation and needs.

Implementing crate: `crates/species/src/`

### Referenced Source
- `crates/engine/src/engine.rs:445`
- `crates/engine/src/engine.rs:2163`

### Test Coverage
- `crates/economy/src/stocks.rs:285`
- `crates/economy/src/stocks.rs:452`
- `crates/economy/tests/fr_matrix_batch7.rs:8`
- `crates/economy/tests/fr_matrix_batch7.rs:46`
- `crates/economy/tests/fr_matrix_batch7.rs:49`
- `crates/economy/tests/fr_matrix_batch7.rs:61`
- `crates/economy/tests/fr_matrix_batch7.rs:77`
- `crates/economy/tests/fr_matrix_batch7.rs:106`

## Decision

TBD -- The architectural decision for FR-CIV-LIFE-020 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/species/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Life simulation and needs requirement in the simulation

### Negative
- Adds complexity to the species crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/species/`
2. **Option B**: Extract into a dedicated sub-crate
