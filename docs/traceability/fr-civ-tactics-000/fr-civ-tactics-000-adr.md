# ADR: FR-CIV-TACTICS-000 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-000
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-000 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:85`
- `docs/development-guide/p-w1-kickoff.md:20`

### Test Coverage
- `crates/tactics/src/lib.rs:208`
- `crates/tactics/tests/fr_matrix_batch2.rs:36`
- `crates/tactics/tests/fr_matrix_batch2.rs:39`
- `crates/tactics/tests/fr_matrix_batch2.rs:40`
- `crates/tactics/tests/fr_matrix_batch5.rs:7`
- `crates/tactics/tests/fr_matrix_batch5.rs:76`
- `crates/tactics/tests/fr_matrix_batch5.rs:77`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-000 needs to be finalized based on implementation exploration.

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
