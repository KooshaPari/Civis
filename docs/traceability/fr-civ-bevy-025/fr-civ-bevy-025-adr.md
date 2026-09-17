# ADR: FR-CIV-BEVY-025 -- Bevy rendering client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BEVY-025
> Epic: FR-CIV-BEVY

## Context

FR-CIV-BEVY-025 is part of the FR-CIV-BEVY epic. This functional requirement captures: Bevy rendering client.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `clients/bevy-ref/README.md:26`
- `docs/development-guide/p-w1-kickoff.md:136`
- `justfile:149`

### Test Coverage
- `clients/bevy-ref/src/live_pick.rs:286`
- `clients/bevy-ref/src/live_pick.rs:306`
- `clients/bevy-ref/src/live_stream.rs:712`
- `crates/build/tests/fr_matrix_batch12.rs:338`
- `crates/build/tests/fr_matrix_batch12.rs:341`

## Decision

TBD -- The architectural decision for FR-CIV-BEVY-025 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Bevy rendering client requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
