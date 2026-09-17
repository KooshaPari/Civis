# ADR: FR-CIV-GEO-003 -- Geological systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GEO-003
> Epic: FR-CIV-GEO

## Context

FR-CIV-GEO-003 is part of the FR-CIV-GEO epic. This functional requirement captures: Geological systems.

Implementing crate: `crates/planet/src/`

### Referenced Source
- `docs/specs/CIV-0300-rts-ui-ux-spec.md:2026`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-GEO-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/planet/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Geological systems requirement in the simulation

### Negative
- Adds complexity to the planet crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/planet/`
2. **Option B**: Extract into a dedicated sub-crate
