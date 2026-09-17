# ADR: FR-REPLAY-001 -- Replay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-REPLAY-001
> Epic: FR-REPLAY

## Context

FR-REPLAY-001 is part of the FR-REPLAY epic. This functional requirement captures: Replay.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `crates/engine/src/engine.rs:1268`
- `crates/engine/src/replay_format.rs:1`
- `docs/reference/agileplus-artifacts-index.md:239`
- `docs/reference/agileplus-artifacts-index.md:311`

### Test Coverage
- `crates/engine/src/engine.rs:3206`
- `crates/engine/tests/fr_matrix_batch1.rs:15`
- `crates/engine/tests/fr_matrix_batch1.rs:191`
- `crates/engine/tests/fr_matrix_batch1.rs:192`
- `crates/engine/tests/fr_matrix_batch1.rs:193`
- `crates/engine/tests/fr_matrix_batch3.rs:87`
- `crates/engine/tests/fr_matrix_batch3.rs:88`

## Decision

TBD -- The architectural decision for FR-REPLAY-001 needs to be finalized based on implementation exploration.

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
