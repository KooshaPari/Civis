# ADR: FR-CORE-009 -- Core system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CORE-009
> Epic: FR-CORE

## Context

FR-CORE-009 is part of the FR-CORE epic. This functional requirement captures: Core system.

Implementing crate: `crates/engine/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CORE-009 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core system requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
