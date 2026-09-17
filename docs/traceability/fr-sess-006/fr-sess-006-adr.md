# ADR: FR-SESS-006 -- Session

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SESS-006
> Epic: FR-SESS

## Context

FR-SESS-006 is part of the FR-SESS epic. This functional requirement captures: Session.

Implementing crate: `crates/server/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-SESS-006 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/server/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Session requirement in the simulation

### Negative
- Adds complexity to the server crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/server/`
2. **Option B**: Extract into a dedicated sub-crate
