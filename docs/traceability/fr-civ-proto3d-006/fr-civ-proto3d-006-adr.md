# ADR: FR-CIV-PROTO3D-006 -- Core civilisation simulation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PROTO3D-006
> Epic: FR-CIV

## Context

FR-CIV-PROTO3D-006 is part of the FR-CIV epic. This functional requirement captures: Core civilisation simulation.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/protocol-3d/src/lib.rs:1180`

## Decision

TBD -- The architectural decision for FR-CIV-PROTO3D-006 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core civilisation simulation requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
