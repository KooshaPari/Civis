# ADR: FR-CIV-WEB-004 -- Web client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-WEB-004
> Epic: FR-CIV-WEB

## Context

FR-CIV-WEB-004 is part of the FR-CIV-WEB epic. This functional requirement captures: Web client.

Implementing crate: `crates/server/src/`

### Referenced Source
- `docs/development-guide/fr-web-spectator.md:33`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-WEB-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/server/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Web client requirement in the simulation

### Negative
- Adds complexity to the server crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/server/`
2. **Option B**: Extract into a dedicated sub-crate
