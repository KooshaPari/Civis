# ADR: FR-CIV-GODTOOL-920 -- Civ Godtool

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GODTOOL-920
> Epic: FR-CIV-GODTOOL

## Context

FR-CIV-GODTOOL-920 is part of the FR-CIV-GODTOOL epic. This functional requirement captures: Civ Godtool.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w1-voxel-render.md:12`
- `docs/agileplus/epics/civ-w1-voxel-render.md:22`
- `docs/agileplus/epics/civ-w6-ui.md:10`
- `docs/agileplus/epics/civ-w6-ui.md:24`
- `docs/agileplus/README.md:25`
- `docs/research/spore-black-and-white.md:31`
- `docs/research/worldbox.md:35`
- `docs/specs/requirements/FR-CIV-GODTOOL.md:16`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-GODTOOL-920 needs to be finalized based on implementation exploration.

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
