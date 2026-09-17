# ADR: FR-CIV-PBR-002 -- Physically based rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PBR-002
> Epic: FR-CIV-PBR

## Context

FR-CIV-PBR-002 is part of the FR-CIV-PBR epic. This functional requirement captures: Physically based rendering.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/material_pbr.rs:495`
- `crates/voxel/src/material_pbr.rs:526`

### Test Coverage
- `crates/voxel/src/material_pbr.rs:1145`
- `crates/voxel/src/material_pbr.rs:1147`
- `crates/voxel/src/material_pbr.rs:1175`
- `crates/voxel/src/material_pbr.rs:1195`

## Decision

TBD -- The architectural decision for FR-CIV-PBR-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Physically based rendering requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
