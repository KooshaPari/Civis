# ADR: FR-CIV-PBR-001 -- Physically based rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PBR-001
> Epic: FR-CIV-PBR

## Context

FR-CIV-PBR-001 is part of the FR-CIV-PBR epic. This functional requirement captures: Physically based rendering.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/material_pbr.rs:16`
- `crates/voxel/src/material_pbr.rs:41`

### Test Coverage
- `crates/voxel/src/material_pbr.rs:902`
- `crates/voxel/src/material_pbr.rs:904`
- `crates/voxel/src/material_pbr.rs:930`
- `crates/voxel/src/material_pbr.rs:954`

## Decision

TBD -- The architectural decision for FR-CIV-PBR-001 needs to be finalized based on implementation exploration.

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
