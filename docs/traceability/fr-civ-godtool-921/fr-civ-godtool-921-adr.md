# ADR: FR-CIV-GODTOOL-921 -- Civ Godtool

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GODTOOL-921
> Epic: FR-CIV-GODTOOL

## Context

FR-CIV-GODTOOL-921 is part of the FR-CIV-GODTOOL epic. This functional requirement captures: Civ Godtool.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w1-voxel-render.md:13`
- `docs/agileplus/epics/civ-w1-voxel-render.md:23`
- `docs/agileplus/epics/civ-w6-ui.md:11`
- `docs/agileplus/epics/civ-w6-ui.md:25`
- `docs/agileplus/README.md:25`
- `docs/specs/requirements/FR-CIV-GODTOOL.md:17`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-GODTOOL-921 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Godtool requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
