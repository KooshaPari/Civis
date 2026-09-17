# ADR: FR-CIV-BEVY-015 -- Bevy rendering client

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BEVY-015
> Epic: FR-CIV-BEVY

## Context

FR-CIV-BEVY-015 is part of the FR-CIV-BEVY epic. This functional requirement captures: Bevy rendering client.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/development-guide/p-w1-kickoff.md:126`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-BEVY-015 needs to be finalized based on implementation exploration.

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
