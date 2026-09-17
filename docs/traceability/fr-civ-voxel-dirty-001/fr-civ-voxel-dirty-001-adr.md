# ADR: FR-CIV-VOXEL-DIRTY-001 -- Dirty voxel tracking

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-DIRTY-001
> Epic: FR-CIV-VOXEL-DIRTY

## Context

FR-CIV-VOXEL-DIRTY-001 is part of the FR-CIV-VOXEL-DIRTY epic. This functional requirement captures: Dirty voxel tracking.

Implementing crate: `crates/voxel/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/voxel/src/fluid_ca.rs:1887`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-DIRTY-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/voxel/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Dirty voxel tracking requirement in the simulation

### Negative
- Adds complexity to the voxel crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/voxel/`
2. **Option B**: Extract into a dedicated sub-crate
