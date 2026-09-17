# ADR: FR-CIV-VOXEL-002 -- Voxel rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-002
> Epic: FR-CIV-VOXEL

## Context

FR-CIV-VOXEL-002 is part of the FR-CIV-VOXEL epic. This functional requirement captures: Voxel rendering.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `crates/engine/src/engine.rs:478`
- `crates/engine/src/engine.rs:1308`
- `crates/engine/src/engine.rs:1329`
- `crates/engine/src/engine.rs:1365`
- `docs/development-guide/fr-3d-additions.md:18`
- `docs/guides/voxel-emergent-vision-and-migration.md:183`

### Test Coverage
- `crates/engine/src/engine.rs:2561`
- `crates/engine/src/engine.rs:2605`
- `crates/voxel/src/lib.rs:100`
- `crates/voxel/src/lib.rs:198`
- `crates/voxel/src/lib.rs:199`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-002 needs to be finalized based on implementation exploration.

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
