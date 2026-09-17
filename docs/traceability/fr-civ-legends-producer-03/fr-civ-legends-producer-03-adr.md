# ADR: FR-CIV-LEGENDS-PRODUCER-03 -- Civ Legends Producer

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-PRODUCER-03
> Epic: FR-CIV-LEGENDS-PRODUCER

## Context

FR-CIV-LEGENDS-PRODUCER-03 is part of the FR-CIV-LEGENDS-PRODUCER epic. This functional requirement captures: Civ Legends Producer.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `docs/design/legends-engine.md:438`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-PRODUCER-03 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Legends Producer requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
