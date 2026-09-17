# ADR: FR-CIV-CORE-012 -- Core simulation engine

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CORE-012
> Epic: FR-CIV-CORE

## Context

FR-CIV-CORE-012 is part of the FR-CIV-CORE epic. This functional requirement captures: Core simulation engine.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/specs/CIV-0001-core-simulation-loop.md:922`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-CORE-012 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core simulation engine requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
