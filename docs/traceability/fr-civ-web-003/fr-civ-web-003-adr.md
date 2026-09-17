# ADR: FR-CIV-WEB-003 -- Web client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-WEB-003
> Epic: FR-CIV-WEB

## Context

FR-CIV-WEB-003 is part of the FR-CIV-WEB epic. This functional requirement captures: Web client.

Implementing crate: `crates/server/src/`

### Referenced Source
- `crates/engine/src/spectator.rs:1`
- `docs/development-guide/fr-web-spectator.md:32`
- `docs/development-guide/fr-web-spectator.md:36`
- `web/src/snapshotView.mjs:2`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-WEB-003 needs to be finalized based on implementation exploration.

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
