# ADR: FR-CIV-SAVE-001 -- Save/load persistence

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SAVE-001
> Epic: FR-CIV-SAVE

## Context

FR-CIV-SAVE-001 is part of the FR-CIV-SAVE epic. This functional requirement captures: Save/load persistence.

Implementing crate: `crates/save-db/src/`

### Referenced Source
- `crates/server/src/saves.rs:26`
- `crates/server/src/saves.rs:99`
- `crates/server/src/saves.rs:276`

### Test Coverage
- `crates/server/src/jsonrpc.rs:2799`
- `crates/server/src/jsonrpc.rs:2832`
- `crates/server/src/saves.rs:463`
- `crates/server/src/saves.rs:465`
- `crates/server/src/saves.rs:472`
- `crates/server/src/saves.rs:499`
- `crates/server/src/saves.rs:544`
- `crates/server/src/saves.rs:556`

## Decision

TBD -- The architectural decision for FR-CIV-SAVE-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/save-db/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Save/load persistence requirement in the simulation

### Negative
- Adds complexity to the save-db crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/save-db/`
2. **Option B**: Extract into a dedicated sub-crate
