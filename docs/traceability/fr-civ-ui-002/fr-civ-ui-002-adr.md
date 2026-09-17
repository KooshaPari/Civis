# ADR: FR-CIV-UI-002 -- User interface

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-UI-002
> Epic: FR-CIV-UI

## Context

FR-CIV-UI-002 is part of the FR-CIV-UI epic. This functional requirement captures: User interface.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/guides/voxel-emergent-vision-and-migration.md:99`
- `docs/guides/voxel-emergent-vision-and-migration.md:160`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-UI-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the User interface requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
