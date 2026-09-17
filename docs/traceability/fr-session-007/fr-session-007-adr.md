# ADR: FR-SESSION-007 -- Session management

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SESSION-007
> Epic: FR-SESSION

## Context

FR-SESSION-007 is part of the FR-SESSION epic. This functional requirement captures: Session management.

Implementing crate: `crates/server/src/`

### Referenced Source
- `docs/specs/CIV-0900-pve-session-and-ai-opponent-spec.md:2034`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-SESSION-007 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/server/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Session management requirement in the simulation

### Negative
- Adds complexity to the server crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/server/`
2. **Option B**: Extract into a dedicated sub-crate
