# ADR: FR-CIV-ACTOR-001 -- Actor and citizen lifecycle

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ACTOR-001
> Epic: FR-CIV-ACTOR

## Context

FR-CIV-ACTOR-001 is part of the FR-CIV-ACTOR epic. This functional requirement captures: Actor and citizen lifecycle.

Implementing crate: `crates/species/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:73`
- `docs/reference/agileplus-artifacts-index.md:268`
- `PLAN.md:145`
- `PLAN.md:146`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:135`
- `crates/build/tests/fr_matrix_batch12.rs:138`

## Decision

TBD -- The architectural decision for FR-CIV-ACTOR-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/species/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Actor and citizen lifecycle requirement in the simulation

### Negative
- Adds complexity to the species crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/species/`
2. **Option B**: Extract into a dedicated sub-crate
