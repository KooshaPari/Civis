# ADR: FR-CIV-RESEARCH-003-EXPORT -- Technology research

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RESEARCH-003-EXPORT
> Epic: FR-CIV-RESEARCH

## Context

FR-CIV-RESEARCH-003-EXPORT is part of the FR-CIV-RESEARCH epic. This functional requirement captures: Technology research.

Implementing crate: `crates/research/src/`

### Referenced Source
- `PLAN.md:237`
- `PLAN.md:238`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-RESEARCH-003-EXPORT needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/research/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Technology research requirement in the simulation

### Negative
- Adds complexity to the research crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/research/`
2. **Option B**: Extract into a dedicated sub-crate
