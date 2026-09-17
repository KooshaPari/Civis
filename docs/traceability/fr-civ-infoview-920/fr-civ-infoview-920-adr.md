# ADR: FR-CIV-INFOVIEW-920 -- Info views and inspectors

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INFOVIEW-920
> Epic: FR-CIV-INFOVIEW

## Context

FR-CIV-INFOVIEW-920 is part of the FR-CIV-INFOVIEW epic. This functional requirement captures: Info views and inspectors.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w4-perception.md:16`
- `docs/agileplus/epics/civ-w4-perception.md:34`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:115`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:19`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-INFOVIEW-920 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Info views and inspectors requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
