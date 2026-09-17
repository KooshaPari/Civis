# ADR: FR-CIV-VOXEL-021 -- Voxel rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-021
> Epic: FR-CIV-VOXEL

## Context

FR-CIV-VOXEL-021 is part of the FR-CIV-VOXEL epic. This functional requirement captures: Voxel rendering.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `crates/voxel/src/worldgen.rs:540`
- `docs/guides/voxel-emergent-vision-and-migration.md:94`
- `docs/guides/voxel-emergent-vision-and-migration.md:124`

### Test Coverage
- `crates/voxel/src/worldgen.rs:543`
- `crates/voxel/src/worldgen.rs:556`
- `crates/voxel/src/worldgen.rs:571`
- `crates/voxel/src/worldgen.rs:591`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-021 needs to be finalized based on implementation exploration.

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
