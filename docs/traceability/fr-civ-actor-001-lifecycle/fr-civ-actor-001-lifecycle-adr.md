# ADR: FR-CIV-ACTOR-001-LIFECYCLE -- Actor and citizen lifecycle

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ACTOR-001-LIFECYCLE
> Epic: FR-CIV-ACTOR

## Context

FR-CIV-ACTOR-001-LIFECYCLE is part of the FR-CIV-ACTOR epic. This functional requirement captures: Actor and citizen lifecycle.

Implementing crate: `crates/species/src/`

### Referenced Source
- `PLAN.md:145`
- `PLAN.md:146`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:147`
- `crates/build/tests/fr_matrix_batch12.rs:150`

## Decision

TBD -- The architectural decision for FR-CIV-ACTOR-001-LIFECYCLE needs to be finalized based on implementation exploration.

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
