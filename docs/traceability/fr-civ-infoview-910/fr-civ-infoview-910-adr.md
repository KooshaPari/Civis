# ADR: FR-CIV-INFOVIEW-910 -- Info views and inspectors

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INFOVIEW-910
> Epic: FR-CIV-INFOVIEW

## Context

FR-CIV-INFOVIEW-910 is part of the FR-CIV-INFOVIEW epic. This functional requirement captures: Info views and inspectors.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/bevy-ref/src/info_views.rs:19`
- `clients/bevy-ref/src/info_views.rs:268`
- `docs/agileplus/epics/civ-w4-perception.md:11`
- `docs/agileplus/epics/civ-w4-perception.md:29`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:105`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:14`

### Test Coverage
- `clients/bevy-ref/src/info_views.rs:796`
- `clients/bevy-ref/src/info_views.rs:805`
- `clients/bevy-ref/src/info_views.rs:817`

## Decision

TBD -- The architectural decision for FR-CIV-INFOVIEW-910 needs to be finalized based on implementation exploration.

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
