# ADR: FR-CIV-VEHICLE-040 -- Vehicle systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VEHICLE-040
> Epic: FR-CIV-VEHICLE

## Context

FR-CIV-VEHICLE-040 is part of the FR-CIV-VEHICLE epic. This functional requirement captures: Vehicle systems.

Implementing crate: `crates/civ-traffic/src/`

### Referenced Source
- `docs/design/vehicles-logistics.md:277`
- `docs/design/vehicles-logistics.md:278`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-VEHICLE-040 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-traffic/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Vehicle systems requirement in the simulation

### Negative
- Adds complexity to the civ-traffic crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-traffic/`
2. **Option B**: Extract into a dedicated sub-crate
