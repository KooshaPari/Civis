# ADR: FR-ECON-008 -- Economics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-ECON-008
> Epic: FR-ECON

## Context

FR-ECON-008 is part of the FR-ECON epic. This functional requirement captures: Economics.

Implementing crate: `crates/economy/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-ECON-008 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Economics requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
