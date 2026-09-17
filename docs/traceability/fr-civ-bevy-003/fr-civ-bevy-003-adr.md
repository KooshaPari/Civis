# ADR: FR-CIV-BEVY-003 -- Bevy rendering client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BEVY-003
> Epic: FR-CIV-BEVY

## Context

FR-CIV-BEVY-003 is part of the FR-CIV-BEVY epic. This functional requirement captures: Bevy rendering client.

Implementing crate: `crates/engine/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `clients/bevy-ref/src/lib.rs:1300`

## Decision

TBD -- The architectural decision for FR-CIV-BEVY-003 needs to be finalized based on implementation exploration.

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
