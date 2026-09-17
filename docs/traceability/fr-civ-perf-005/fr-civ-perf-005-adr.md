# ADR: FR-CIV-PERF-005 -- Performance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PERF-005
> Epic: FR-CIV-PERF

## Context

FR-CIV-PERF-005 is part of the FR-CIV-PERF epic. This functional requirement captures: Performance.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/specs/CIV-0500-performance-optimization-spec.md:1941`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-PERF-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Performance requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
