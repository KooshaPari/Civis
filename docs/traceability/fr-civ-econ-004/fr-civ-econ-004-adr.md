# ADR: FR-CIV-ECON-004 -- Economy and joule allocation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ECON-004
> Epic: FR-CIV-ECON

## Context

FR-CIV-ECON-004 is part of the FR-CIV-ECON epic. This functional requirement captures: Economy and joule allocation.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/reference/CODE_ENTITY_MAP.md:8`
- `docs/reference/FR_TRACKER.md:10`
- `docs/reports/STATUS_REPORT.md:92`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ECON-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Economy and joule allocation requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
