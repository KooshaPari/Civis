# ADR: FR-CIV-WAR-042 -- War and conflict

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-WAR-042
> Epic: FR-CIV-WAR

## Context

FR-CIV-WAR-042 is part of the FR-CIV-WAR epic. This functional requirement captures: War and conflict.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `docs/design/warfare.md:150`
- `docs/design/warfare.md:201`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-WAR-042 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/tactics/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the War and conflict requirement in the simulation

### Negative
- Adds complexity to the tactics crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/tactics/`
2. **Option B**: Extract into a dedicated sub-crate
