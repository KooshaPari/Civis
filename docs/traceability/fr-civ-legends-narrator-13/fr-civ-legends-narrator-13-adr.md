# ADR: FR-CIV-LEGENDS-NARRATOR-13 -- Civ Legends Narrator

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-NARRATOR-13
> Epic: FR-CIV-LEGENDS-NARRATOR

## Context

FR-CIV-LEGENDS-NARRATOR-13 is part of the FR-CIV-LEGENDS-NARRATOR epic. This functional requirement captures: Civ Legends Narrator.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `docs/design/legends-engine.md:448`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-NARRATOR-13 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Legends Narrator requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
