# ADR: FR-CIV-VOXEL-010 -- Voxel rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-010
> Epic: FR-CIV-VOXEL

## Context

FR-CIV-VOXEL-010 is part of the FR-CIV-VOXEL epic. This functional requirement captures: Voxel rendering.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:24`
- `docs/guides/voxel-emergent-vision-and-migration.md:29`
- `docs/guides/voxel-emergent-vision-and-migration.md:96`
- `docs/worklogs/2026-05-22-civis-3d-kickoff.md:72`

### Test Coverage
- `crates/voxel/src/lib.rs:118`
- `crates/voxel/src/lib.rs:261`
- `crates/voxel/src/lib.rs:262`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-010 needs to be finalized based on implementation exploration.

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
