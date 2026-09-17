# ADR: FR-CIV-CA-009 -- Combat and military

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CA-009
> Epic: FR-CIV-CA

## Context

FR-CIV-CA-009 is part of the FR-CIV-CA epic. This functional requirement captures: Combat and military.

Implementing crate: `crates/ai/src/`

### Referenced Source
- `crates/engine/src/engine.rs:417`
- `crates/engine/src/engine.rs:1449`
- `crates/engine/src/engine.rs:1501`
- `crates/voxel/src/fluid_ca.rs:124`
- `crates/voxel/src/fluid_ca.rs:141`

### Test Coverage
- `crates/engine/src/engine.rs:3343`
- `crates/engine/src/engine.rs:3346`
- `crates/engine/src/engine.rs:3356`
- `crates/voxel/src/fluid_ca.rs:2105`
- `crates/voxel/src/fluid_ca.rs:2110`

## Decision

TBD -- The architectural decision for FR-CIV-CA-009 needs to be finalized based on implementation exploration.

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
