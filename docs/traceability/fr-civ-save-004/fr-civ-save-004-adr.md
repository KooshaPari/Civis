# ADR: FR-CIV-SAVE-004 -- Save/load persistence

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SAVE-004
> Epic: FR-CIV-SAVE

## Context

FR-CIV-SAVE-004 is part of the FR-CIV-SAVE epic. This functional requirement captures: Save/load persistence.

Implementing crate: `crates/save-db/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-SAVE-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/save-db/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Save/load persistence requirement in the simulation

### Negative
- Adds complexity to the save-db crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/save-db/`
2. **Option B**: Extract into a dedicated sub-crate
