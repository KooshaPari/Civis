# ADR: FR-CIV-LAWS-004 -- Legal system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LAWS-004
> Epic: FR-CIV-LAWS

## Context

FR-CIV-LAWS-004 is part of the FR-CIV-LAWS epic. This functional requirement captures: Legal system.

Implementing crate: `crates/laws/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/laws/src/lib.rs:264`

## Decision

TBD -- The architectural decision for FR-CIV-LAWS-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/laws/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Legal system requirement in the simulation

### Negative
- Adds complexity to the laws crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/laws/`
2. **Option B**: Extract into a dedicated sub-crate
