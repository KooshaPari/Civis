# ADR: FR-CIV-INFOVIEW-912 -- Info views and inspectors

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INFOVIEW-912
> Epic: FR-CIV-INFOVIEW

## Context

FR-CIV-INFOVIEW-912 is part of the FR-CIV-INFOVIEW epic. This functional requirement captures: Info views and inspectors.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w4-perception.md:13`
- `docs/agileplus/epics/civ-w4-perception.md:31`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:107`
- `docs/research/songs-of-syx.md:34`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:16`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-INFOVIEW-912 needs to be finalized based on implementation exploration.

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
