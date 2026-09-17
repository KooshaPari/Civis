# ADR: FR-CIV-0001 -- Core civilisation simulation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-0001
> Epic: FR-CIV

## Context

FR-CIV-0001 is part of the FR-CIV epic. This functional requirement captures: Core civilisation simulation.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/guides/GIT_WORKTREE_GUIDE.md:151`
- `PLAN.md:16`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:102`
- `crates/build/tests/fr_matrix_batch12.rs:105`

## Decision

TBD -- The architectural decision for FR-CIV-0001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core civilisation simulation requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
