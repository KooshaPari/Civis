# ADR: FR-GUARD-002 -- Guard rails

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-GUARD-002
> Epic: FR-GUARD

## Context

FR-GUARD-002 is part of the FR-GUARD epic. This functional requirement captures: Guard rails.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1337`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-GUARD-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Guard rails requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
