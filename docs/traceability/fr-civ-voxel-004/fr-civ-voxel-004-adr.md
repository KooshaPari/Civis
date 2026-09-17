# ADR: FR-CIV-VOXEL-004 -- Voxel rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-004
> Epic: FR-CIV-VOXEL

## Context

FR-CIV-VOXEL-004 is part of the FR-CIV-VOXEL epic. This functional requirement captures: Voxel rendering.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:22`

### Test Coverage
- `crates/voxel/src/lib.rs:251`
- `crates/voxel/src/lib.rs:252`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/voxel/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Voxel rendering requirement in the simulation

### Negative
- Adds complexity to the voxel crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/voxel/`
2. **Option B**: Extract into a dedicated sub-crate
