# ADR: FR-CIV-HUD-001 -- HUD overlay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-HUD-001
> Epic: FR-CIV-HUD

## Context

FR-CIV-HUD-001 is part of the FR-CIV-HUD epic. This functional requirement captures: HUD overlay.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `crates/voxel/src/hud.rs:5`
- `crates/voxel/src/hud.rs:7`
- `crates/voxel/src/hud.rs:32`
- `docs/reference/agileplus-artifacts-index.md:201`
- `docs/reference/agileplus-artifacts-index.md:300`

### Test Coverage
- `crates/voxel/src/hud.rs:729`
- `crates/voxel/src/hud.rs:731`
- `crates/voxel/src/hud.rs:748`

## Decision

TBD -- The architectural decision for FR-CIV-HUD-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the HUD overlay requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
