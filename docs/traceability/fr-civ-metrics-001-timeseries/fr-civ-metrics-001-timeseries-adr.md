# ADR: FR-CIV-METRICS-001-TIMESERIES -- Metrics and monitoring

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-METRICS-001-TIMESERIES
> Epic: FR-CIV-METRICS

## Context

FR-CIV-METRICS-001-TIMESERIES is part of the FR-CIV-METRICS epic. This functional requirement captures: Metrics and monitoring.

Implementing crate: `crates/observability/src/`

### Referenced Source
- `PLAN.md:151`
- `PLAN.md:152`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-METRICS-001-TIMESERIES needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/observability/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Metrics and monitoring requirement in the simulation

### Negative
- Adds complexity to the observability crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/observability/`
2. **Option B**: Extract into a dedicated sub-crate
