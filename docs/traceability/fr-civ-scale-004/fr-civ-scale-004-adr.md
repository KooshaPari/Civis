# ADR: FR-CIV-SCALE-004 -- Scale and performance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SCALE-004
> Epic: FR-CIV-SCALE

## Context

FR-CIV-SCALE-004 is part of the FR-CIV-SCALE epic. This functional requirement captures: Scale and performance.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/scale_budget.rs:18`
- `crates/voxel/src/scale_budget.rs:667`
- `crates/voxel/src/scale_budget.rs:798`
- `crates/voxel/src/scale_budget.rs:799`
- `docs/design/streaming-window.md:150`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:1113`
- `crates/voxel/src/scale_budget.rs:1115`
- `crates/voxel/src/scale_budget.rs:1127`
- `crates/voxel/src/scale_budget.rs:1156`
- `crates/voxel/src/scale_budget.rs:1174`
- `crates/voxel/src/scale_budget.rs:1201`

## Decision

TBD -- The architectural decision for FR-CIV-SCALE-004 needs to be finalized based on implementation exploration.

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
