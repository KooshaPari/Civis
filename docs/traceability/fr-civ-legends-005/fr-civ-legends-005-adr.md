# ADR: FR-CIV-LEGENDS-005 -- Legend and narrative system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-005
> Epic: FR-CIV-LEGENDS

## Context

FR-CIV-LEGENDS-005 is part of the FR-CIV-LEGENDS epic. This functional requirement captures: Legend and narrative system.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/legends/src/graph.rs:163`
- `crates/legends/src/query.rs:85`
- `crates/legends/src/query.rs:92`
- `crates/legends/src/query.rs:119`
- `crates/legends/src/query.rs:137`
- `crates/legends/src/query.rs:145`
- `crates/legends/src/query.rs:217`
- `crates/legends/src/query.rs:237`

### Test Coverage
- `crates/legends/tests/fr_legends_completion.rs:7`
- `crates/legends/tests/fr_legends_completion.rs:287`
- `crates/legends/tests/fr_legends_completion.rs:290`
- `crates/legends/tests/fr_legends_completion.rs:367`

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-005 needs to be finalized based on implementation exploration.

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
