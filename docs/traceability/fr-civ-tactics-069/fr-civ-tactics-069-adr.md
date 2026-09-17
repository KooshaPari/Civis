# ADR: FR-CIV-TACTICS-069 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-069
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-069 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `docs/development-guide/p-w1-kickoff.md:71`

### Test Coverage
- `crates/mod-host/tests/fr_matrix_batch10.rs:14`
- `crates/mod-host/tests/fr_matrix_batch10.rs:184`
- `crates/mod-host/tests/fr_matrix_batch10.rs:185`
- `crates/mod-host/tests/fr_matrix_batch10.rs:186`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-069 needs to be finalized based on implementation exploration.

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
