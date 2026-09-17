# ADR: FR-CIV-SOCIAL-002 -- Social systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SOCIAL-002
> Epic: FR-CIV-SOCIAL

## Context

FR-CIV-SOCIAL-002 is part of the FR-CIV-SOCIAL epic. This functional requirement captures: Social systems.

Implementing crate: `crates/civ-institutions/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:73`
- `docs/reference/agileplus-artifacts-index.md:271`
- `PLAN.md:149`
- `PLAN.md:150`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-SOCIAL-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-institutions/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Social systems requirement in the simulation

### Negative
- Adds complexity to the civ-institutions crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-institutions/`
2. **Option B**: Extract into a dedicated sub-crate
