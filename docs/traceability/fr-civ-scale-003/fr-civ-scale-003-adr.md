# ADR: FR-CIV-SCALE-003 -- Scale and performance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SCALE-003
> Epic: FR-CIV-SCALE

## Context

FR-CIV-SCALE-003 is part of the FR-CIV-SCALE epic. This functional requirement captures: Scale and performance.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/voxel/src/scale_budget.rs:14`
- `crates/voxel/src/scale_budget.rs:404`
- `crates/voxel/src/scale_budget.rs:494`
- `crates/voxel/src/scale_budget.rs:495`
- `docs/design/streaming-window.md:225`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:1033`
- `crates/voxel/src/scale_budget.rs:1035`
- `crates/voxel/src/scale_budget.rs:1058`
- `crates/voxel/src/scale_budget.rs:1077`
- `crates/voxel/src/scale_budget.rs:1095`

## Decision

TBD -- The architectural decision for FR-CIV-SCALE-003 needs to be finalized based on implementation exploration.

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
