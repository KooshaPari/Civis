# ADR: FR-CIV-SCALE-002 -- Scale and performance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SCALE-002
> Epic: FR-CIV-SCALE

## Context

FR-CIV-SCALE-002 is part of the FR-CIV-SCALE epic. This functional requirement captures: Scale and performance.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/scale_budget.rs:11`
- `crates/voxel/src/scale_budget.rs:277`
- `crates/voxel/src/scale_budget.rs:284`
- `crates/voxel/src/scale_budget.rs:300`
- `crates/voxel/src/scale_budget.rs:305`
- `crates/voxel/src/scale_budget.rs:306`
- `crates/voxel/src/scale_budget.rs:317`
- `crates/voxel/src/scale_budget.rs:367`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:970`
- `crates/voxel/src/scale_budget.rs:972`
- `crates/voxel/src/scale_budget.rs:985`
- `crates/voxel/src/scale_budget.rs:1006`
- `crates/voxel/src/scale_budget.rs:1025`

## Decision

TBD -- The architectural decision for FR-CIV-SCALE-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Scale and performance requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
