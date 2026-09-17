# ADR: FR-CIV-CORE-DET-002 -- Determinism guarantees

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CORE-DET-002
> Epic: FR-CIV-CORE-DET

## Context

FR-CIV-CORE-DET-002 is part of the FR-CIV-CORE-DET epic. This functional requirement captures: Determinism guarantees.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3214`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-CORE-DET-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Determinism guarantees requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
