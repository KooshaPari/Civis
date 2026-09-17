# ADR: FR-ECO-003 -- Economy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-ECO-003
> Epic: FR-ECO

## Context

FR-ECO-003 is part of the FR-ECO epic. This functional requirement captures: Economy.

Implementing crate: `crates/economy/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/economy/src/lib.rs:246`
- `docs/specs/CIV-0100-economy-v1.md:1682`

## Decision

TBD -- The architectural decision for FR-ECO-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Economy requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
