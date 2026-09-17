# ADR: FR-CIV-LEGENDS-007 -- Legend and narrative system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-007
> Epic: FR-CIV-LEGENDS

## Context

FR-CIV-LEGENDS-007 is part of the FR-CIV-LEGENDS epic. This functional requirement captures: Legend and narrative system.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/legends/src/rumor.rs:55`
- `crates/legends/src/rumor.rs:113`
- `crates/legends/src/rumor.rs:199`

### Test Coverage
- `crates/legends/src/rumor.rs:870`

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-007 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Legend and narrative system requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
