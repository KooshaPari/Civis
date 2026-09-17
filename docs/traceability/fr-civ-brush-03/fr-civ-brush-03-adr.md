# ADR: FR-CIV-BRUSH-03 -- Terrain brush tools

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BRUSH-03
> Epic: FR-CIV-BRUSH

## Context

FR-CIV-BRUSH-03 is part of the FR-CIV-BRUSH epic. This functional requirement captures: Terrain brush tools.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `docs/design/brush-tool-system.md:516`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-BRUSH-03 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/voxel/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Terrain brush tools requirement in the simulation

### Negative
- Adds complexity to the voxel crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/voxel/`
2. **Option B**: Extract into a dedicated sub-crate
