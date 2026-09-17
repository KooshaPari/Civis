# ADR: FR-CIV-RESEARCH-003 -- Technology research

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RESEARCH-003
> Epic: FR-CIV-RESEARCH

## Context

FR-CIV-RESEARCH-003 is part of the FR-CIV-RESEARCH epic. This functional requirement captures: Technology research.

Implementing crate: `crates/research/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:81`
- `PLAN.md:237`
- `PLAN.md:238`

### Test Coverage
- `crates/research/src/lib.rs:619`
- `crates/research/src/lib.rs:620`

## Decision

TBD -- The architectural decision for FR-CIV-RESEARCH-003 needs to be finalized based on implementation exploration.

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
