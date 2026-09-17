# ADR: FR-CIV-TECH-005 -- Technology tree

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TECH-005
> Epic: FR-CIV-TECH

## Context

FR-CIV-TECH-005 is part of the FR-CIV-TECH epic. This functional requirement captures: Technology tree.

Implementing crate: `crates/research/src/`

### Referenced Source
- `docs/design/tech-engineering.md:229`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-TECH-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/research/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Technology tree requirement in the simulation

### Negative
- Adds complexity to the research crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/research/`
2. **Option B**: Extract into a dedicated sub-crate
