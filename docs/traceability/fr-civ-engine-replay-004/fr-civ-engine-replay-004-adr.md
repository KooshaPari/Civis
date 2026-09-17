# ADR: FR-CIV-ENGINE-REPLAY-004 -- Civ Engine Replay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ENGINE-REPLAY-004
> Epic: FR-CIV-ENGINE-REPLAY

## Context

FR-CIV-ENGINE-REPLAY-004 is part of the FR-CIV-ENGINE-REPLAY epic. This functional requirement captures: Civ Engine Replay.

Implementing crate: `crates/engine/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/engine/src/engine.rs:3172`

## Decision

TBD -- The architectural decision for FR-CIV-ENGINE-REPLAY-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Engine Replay requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
