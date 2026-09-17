# ADR: FR-CIV-CA-002 -- Combat and military

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CA-002
> Epic: FR-CIV-CA

## Context

FR-CIV-CA-002 is part of the FR-CIV-CA epic. This functional requirement captures: Combat and military.

Implementing crate: `crates/ai/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/voxel/src/fluid_ca.rs:1473`
- `crates/voxel/src/fluid_ca.rs:1493`
- `crates/voxel/src/fluid_ca.rs:1508`
- `crates/voxel/src/fluid_ca.rs:1661`
- `crates/voxel/src/fluid_ca.rs:1809`
- `crates/voxel/src/fluid_ca.rs:1854`

## Decision

TBD -- The architectural decision for FR-CIV-CA-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/ai/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Combat and military requirement in the simulation

### Negative
- Adds complexity to the ai crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/ai/`
2. **Option B**: Extract into a dedicated sub-crate
