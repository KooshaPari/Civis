# ADR: FR-CIV-HUD-004 -- HUD overlay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-HUD-004
> Epic: FR-CIV-HUD

## Context

FR-CIV-HUD-004 is part of the FR-CIV-HUD epic. This functional requirement captures: HUD overlay.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `crates/voxel/src/hud.rs:10`
- `crates/voxel/src/hud.rs:509`
- `docs/reference/agileplus-artifacts-index.md:201`
- `docs/reference/agileplus-artifacts-index.md:303`

### Test Coverage
- `crates/voxel/src/hud.rs:858`
- `crates/voxel/src/hud.rs:860`
- `crates/voxel/src/hud.rs:872`
- `crates/voxel/src/hud.rs:889`

## Decision

TBD -- The architectural decision for FR-CIV-HUD-004 needs to be finalized based on implementation exploration.

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
