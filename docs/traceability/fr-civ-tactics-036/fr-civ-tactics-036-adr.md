# ADR: FR-CIV-TACTICS-036 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-036
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-036 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `crates/tactics/src/grid_obstacles.rs:1`
- `crates/tactics/src/pathfinding.rs:17`
- `docs/development-guide/p-w1-kickoff.md:36`

### Test Coverage
- `crates/tactics/src/lib.rs:392`
- `crates/tactics/tests/fr_matrix_batch9.rs:3`
- `crates/tactics/tests/fr_matrix_batch9.rs:31`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-036 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/tactics/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Tactics and strategy requirement in the simulation

### Negative
- Adds complexity to the tactics crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/tactics/`
2. **Option B**: Extract into a dedicated sub-crate
