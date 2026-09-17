# ADR: FR-CIV-LEGENDS-QUERY-07 -- Civ Legends Query

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-QUERY-07
> Epic: FR-CIV-LEGENDS-QUERY

## Context

FR-CIV-LEGENDS-QUERY-07 is part of the FR-CIV-LEGENDS-QUERY epic. This functional requirement captures: Civ Legends Query.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/engine/src/emergence.rs:38`
- `crates/engine/src/emergence.rs:629`
- `docs/design/legends-engine.md:442`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-QUERY-07 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Legends Query requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
