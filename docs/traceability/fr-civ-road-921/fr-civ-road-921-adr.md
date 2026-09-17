# ADR: FR-CIV-ROAD-921 -- Road and path systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ROAD-921
> Epic: FR-CIV-ROAD

## Context

FR-CIV-ROAD-921 is part of the FR-CIV-ROAD epic. This functional requirement captures: Road and path systems.

Implementing crate: `crates/civ-traffic/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w3-infrastructure.md:14`
- `docs/agileplus/epics/civ-w3-infrastructure.md:25`
- `docs/agileplus/README.md:22`
- `docs/research/songs-of-syx.md:37`
- `docs/specs/requirements/FR-CIV-ROAD.md:16`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ROAD-921 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-traffic/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Road and path systems requirement in the simulation

### Negative
- Adds complexity to the civ-traffic crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-traffic/`
2. **Option B**: Extract into a dedicated sub-crate
