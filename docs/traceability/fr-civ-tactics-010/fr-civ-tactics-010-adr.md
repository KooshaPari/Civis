# ADR: FR-CIV-TACTICS-010 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-010
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-010 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `crates/engine/src/engine.rs:436`
- `docs/development-guide/fr-3d-additions.md:87`
- `docs/development-guide/p-w1-kickoff.md:14`
- `docs/development-guide/p-w1-kickoff.md:22`

### Test Coverage
- `crates/engine/src/engine.rs:2670`
- `crates/tactics/src/lib.rs:262`
- `crates/tactics/tests/fr_matrix_batch2.rs:78`
- `crates/tactics/tests/fr_matrix_batch2.rs:81`
- `crates/tactics/tests/fr_matrix_batch2.rs:83`
- `crates/tactics/tests/fr_matrix_batch5.rs:9`
- `crates/tactics/tests/fr_matrix_batch5.rs:105`
- `crates/tactics/tests/fr_matrix_batch5.rs:106`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-010 needs to be finalized based on implementation exploration.

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
