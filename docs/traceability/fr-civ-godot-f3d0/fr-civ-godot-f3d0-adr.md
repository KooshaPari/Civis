# ADR: FR-CIV-GODOT-F3D0 -- Godot client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GODOT-F3D0
> Epic: FR-CIV-GODOT

## Context

FR-CIV-GODOT-F3D0 is part of the FR-CIV-GODOT epic. This functional requirement captures: Godot client.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `clients/godot-ref/rust/src/ws_frame.rs:174`

## Decision

TBD -- The architectural decision for FR-CIV-GODOT-F3D0 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Godot client requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
