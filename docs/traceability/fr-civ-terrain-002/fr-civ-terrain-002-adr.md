# ADR: FR-CIV-TERRAIN-002 -- Civ Terrain

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TERRAIN-002
> Epic: FR-CIV-TERRAIN

## Context

FR-CIV-TERRAIN-002 is part of the FR-CIV-TERRAIN epic. This functional requirement captures: Civ Terrain.

Implementing crate: `crates/planet/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-TERRAIN-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/planet/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Terrain requirement in the simulation

### Negative
- Adds complexity to the planet crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/planet/`
2. **Option B**: Extract into a dedicated sub-crate
