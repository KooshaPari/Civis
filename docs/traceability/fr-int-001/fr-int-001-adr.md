# ADR: FR-INT-001 -- Integration

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-INT-001
> Epic: FR-INT

## Context

FR-INT-001 is part of the FR-INT epic. This functional requirement captures: Integration.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1739`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-INT-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Integration requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
