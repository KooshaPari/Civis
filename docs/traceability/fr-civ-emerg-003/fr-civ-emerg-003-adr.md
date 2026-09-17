# ADR: FR-CIV-EMERG-003 -- Emergence mechanics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-EMERG-003
> Epic: FR-CIV-EMERG

## Context

FR-CIV-EMERG-003 is part of the FR-CIV-EMERG epic. This functional requirement captures: Emergence mechanics.

Implementing crate: `crates/emergence-oracle/src/`

### Referenced Source
- `crates/engine/src/emergence_metrics.rs:251`
- `crates/engine/src/replay.rs:93`
- `crates/engine/src/replay.rs:426`
- `crates/engine/src/replay.rs:702`
- `crates/server/src/jsonrpc.rs:413`
- `crates/server/src/jsonrpc.rs:553`
- `crates/server/src/jsonrpc.rs:774`

### Test Coverage
- `crates/engine/src/replay.rs:763`
- `crates/engine/src/replay.rs:783`
- `crates/server/src/jsonrpc.rs:1972`

## Decision

TBD -- The architectural decision for FR-CIV-EMERG-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/emergence-oracle/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Emergence mechanics requirement in the simulation

### Negative
- Adds complexity to the emergence-oracle crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/emergence-oracle/`
2. **Option B**: Extract into a dedicated sub-crate
