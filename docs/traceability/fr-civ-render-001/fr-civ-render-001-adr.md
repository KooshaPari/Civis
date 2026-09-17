# ADR: FR-CIV-RENDER-001 -- Rendering pipeline

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RENDER-001
> Epic: FR-CIV-RENDER

## Context

FR-CIV-RENDER-001 is part of the FR-CIV-RENDER epic. This functional requirement captures: Rendering pipeline.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/guides/voxel-emergent-vision-and-migration.md:96`
- `docs/guides/voxel-emergent-vision-and-migration.md:148`
- `docs/guides/voxel-emergent-vision-and-migration.md:152`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-RENDER-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Rendering pipeline requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
