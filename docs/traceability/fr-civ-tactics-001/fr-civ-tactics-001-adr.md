# ADR: FR-CIV-TACTICS-001 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-001
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-001 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:86`
- `docs/development-guide/p-w1-kickoff.md:21`

### Test Coverage
- `crates/tactics/src/lib.rs:218`
- `crates/tactics/tests/fr_matrix_batch2.rs:48`
- `crates/tactics/tests/fr_matrix_batch2.rs:51`
- `crates/tactics/tests/fr_matrix_batch2.rs:52`
- `crates/tactics/tests/fr_matrix_batch5.rs:8`
- `crates/tactics/tests/fr_matrix_batch5.rs:86`
- `crates/tactics/tests/fr_matrix_batch5.rs:87`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-001 needs to be finalized based on implementation exploration.

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
