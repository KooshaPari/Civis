# ADR: FR-DET-002 -- Determinism

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-DET-002
> Epic: FR-DET

## Context

FR-DET-002 is part of the FR-DET epic. This functional requirement captures: Determinism.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:290`
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:440`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-DET-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Determinism requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
