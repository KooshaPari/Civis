# ADR: FR-REP-001 -- Replay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-REP-001
> Epic: FR-REP

## Context

FR-REP-001 is part of the FR-REP epic. This functional requirement captures: Replay.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:489`
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:532`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-REP-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Replay requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
