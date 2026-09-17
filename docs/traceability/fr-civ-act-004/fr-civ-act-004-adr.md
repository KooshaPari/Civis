# ADR: FR-CIV-ACT-004 -- Actor lifecycle

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ACT-004
> Epic: FR-CIV-ACT

## Context

FR-CIV-ACT-004 is part of the FR-CIV-ACT epic. This functional requirement captures: Actor lifecycle.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:183`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ACT-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Actor lifecycle requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
