# ADR: FR-CIV-GENETICS-000 -- Procedural genetics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GENETICS-000
> Epic: FR-CIV-GENETICS

## Context

FR-CIV-GENETICS-000 is part of the FR-CIV-GENETICS epic. This functional requirement captures: Procedural genetics.

Implementing crate: `crates/genetics/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:41`

### Test Coverage
- `crates/genetics/src/lib.rs:178`

## Decision

TBD -- The architectural decision for FR-CIV-GENETICS-000 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/genetics/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Procedural genetics requirement in the simulation

### Negative
- Adds complexity to the genetics crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/genetics/`
2. **Option B**: Extract into a dedicated sub-crate
