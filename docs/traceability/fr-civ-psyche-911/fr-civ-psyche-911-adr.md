# ADR: FR-CIV-PSYCHE-911 -- Psychological and social modelling

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PSYCHE-911
> Epic: FR-CIV-PSYCHE

## Context

FR-CIV-PSYCHE-911 is part of the FR-CIV-PSYCHE epic. This functional requirement captures: Psychological and social modelling.

Implementing crate: `crates/needs/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w2-life-sim.md:12`
- `docs/agileplus/epics/civ-w2-life-sim.md:24`
- `docs/agileplus/README.md:21`
- `docs/research/spore-black-and-white.md:32`
- `docs/specs/requirements/FR-CIV-PSYCHE.md:14`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-PSYCHE-911 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/needs/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Psychological and social modelling requirement in the simulation

### Negative
- Adds complexity to the needs crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/needs/`
2. **Option B**: Extract into a dedicated sub-crate
