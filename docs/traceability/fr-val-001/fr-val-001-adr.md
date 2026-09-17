# ADR: FR-VAL-001 -- Validation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-VAL-001
> Epic: FR-VAL

## Context

FR-VAL-001 is part of the FR-VAL epic. This functional requirement captures: Validation.

Implementing crate: `crates/build/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:170`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-VAL-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/build/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Validation requirement in the simulation

### Negative
- Adds complexity to the build crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/build/`
2. **Option B**: Extract into a dedicated sub-crate
