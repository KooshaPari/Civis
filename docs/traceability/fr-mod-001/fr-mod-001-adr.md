# ADR: FR-MOD-001 -- Modding

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-MOD-001
> Epic: FR-MOD

## Context

FR-MOD-001 is part of the FR-MOD epic. This functional requirement captures: Modding.

Implementing crate: `crates/mod-host/src/`

### Referenced Source
- `docs/development-guide/fr-modding-roadmap.md:78`

### Test Coverage
- `crates/mod-host/tests/fr_matrix_batch10.rs:276`

## Decision

TBD -- The architectural decision for FR-MOD-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/mod-host/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Modding requirement in the simulation

### Negative
- Adds complexity to the mod-host crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/mod-host/`
2. **Option B**: Extract into a dedicated sub-crate
