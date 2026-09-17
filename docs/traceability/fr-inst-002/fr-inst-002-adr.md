# ADR: FR-INST-002 -- Institution

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-INST-002
> Epic: FR-INST

## Context

FR-INST-002 is part of the FR-INST epic. This functional requirement captures: Institution.

Implementing crate: `crates/civ-institutions/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-INST-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-institutions/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Institution requirement in the simulation

### Negative
- Adds complexity to the civ-institutions crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-institutions/`
2. **Option B**: Extract into a dedicated sub-crate
