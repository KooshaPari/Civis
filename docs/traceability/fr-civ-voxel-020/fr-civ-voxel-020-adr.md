# ADR: FR-CIV-VOXEL-020 -- Voxel rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-VOXEL-020
> Epic: FR-CIV-VOXEL

## Context

FR-CIV-VOXEL-020 is part of the FR-CIV-VOXEL epic. This functional requirement captures: Voxel rendering.

Implementing crate: `crates/voxel/src/`

### Referenced Source
- `clients/bevy-ref/src/bin/standalone.rs:194`
- `clients/bevy-ref/src/voxel_stream.rs:13`
- `crates/voxel/src/stream.rs:32`
- `docs/guides/voxel-emergent-vision-and-migration.md:94`
- `docs/guides/voxel-emergent-vision-and-migration.md:119`
- `docs/guides/voxel-emergent-vision-and-migration.md:123`

### Test Coverage
- `clients/bevy-ref/src/voxel_stream.rs:352`
- `clients/bevy-ref/src/voxel_stream.rs:364`

## Decision

TBD -- The architectural decision for FR-CIV-VOXEL-020 needs to be finalized based on implementation exploration.

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
