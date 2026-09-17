# ADR: FR-UX-010 -- User experience

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-UX-010
> Epic: FR-UX

## Context

FR-UX-010 is part of the FR-UX epic. This functional requirement captures: User experience.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/models/civ-sim/USER_SPEC.md:942`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-UX-010 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the User experience requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
