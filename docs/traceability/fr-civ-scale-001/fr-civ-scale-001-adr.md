# ADR: FR-CIV-SCALE-001 -- Scale and performance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SCALE-001
> Epic: FR-CIV-SCALE

## Context

FR-CIV-SCALE-001 is part of the FR-CIV-SCALE epic. This functional requirement captures: Scale and performance.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/scale_budget.rs:1`
- `crates/voxel/src/scale_budget.rs:9`
- `crates/voxel/src/scale_budget.rs:57`
- `crates/voxel/src/scale_budget.rs:62`
- `crates/voxel/src/scale_budget.rs:82`
- `crates/voxel/src/scale_budget.rs:84`
- `crates/voxel/src/scale_budget.rs:209`
- `crates/voxel/src/scale_budget.rs:210`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:895`
- `crates/voxel/src/scale_budget.rs:907`
- `crates/voxel/src/scale_budget.rs:909`
- `crates/voxel/src/scale_budget.rs:924`
- `crates/voxel/src/scale_budget.rs:936`
- `crates/voxel/src/scale_budget.rs:951`

## Decision

TBD -- The architectural decision for FR-CIV-SCALE-001 needs to be finalized based on implementation exploration.

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
