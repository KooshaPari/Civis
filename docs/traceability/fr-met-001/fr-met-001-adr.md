# ADR: FR-MET-001 -- Metrics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-MET-001
> Epic: FR-MET

## Context

FR-MET-001 is part of the FR-MET epic. This functional requirement captures: Metrics.

Implementing crate: `crates/observability/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1174`
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1203`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-MET-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/observability/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Metrics requirement in the simulation

### Negative
- Adds complexity to the observability crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/observability/`
2. **Option B**: Extract into a dedicated sub-crate
