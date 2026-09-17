# ADR: FR-CIV-CA-004 -- Combat and military

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CA-004
> Epic: FR-CIV-CA

## Context

FR-CIV-CA-004 is part of the FR-CIV-CA epic. This functional requirement captures: Combat and military.

Implementing crate: `crates/ai/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/voxel/src/fluid_ca.rs:1534`
- `crates/voxel/src/fluid_ca.rs:1671`

## Decision

TBD -- The architectural decision for FR-CIV-CA-004 needs to be finalized based on implementation exploration.

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
