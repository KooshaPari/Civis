# ADR: FR-CIV-GODOT-ATTACH-001 -- Civ Godot Attach

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GODOT-ATTACH-001
> Epic: FR-CIV-GODOT-ATTACH

## Context

FR-CIV-GODOT-ATTACH-001 is part of the FR-CIV-GODOT-ATTACH epic. This functional requirement captures: Civ Godot Attach.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/development-guide/fr-godot-attach.md:9`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-GODOT-ATTACH-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Godot Attach requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
