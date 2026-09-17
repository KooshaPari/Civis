# ADR: FR-CIV-INFOVIEW-900 -- Info views and inspectors

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INFOVIEW-900
> Epic: FR-CIV-INFOVIEW

## Context

FR-CIV-INFOVIEW-900 is part of the FR-CIV-INFOVIEW epic. This functional requirement captures: Info views and inspectors.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/bevy-ref/src/info_views.rs:17`
- `clients/bevy-ref/src/info_views.rs:380`
- `docs/agileplus/epics/civ-w4-perception.md:9`
- `docs/agileplus/epics/civ-w4-perception.md:28`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:214`
- `docs/design/master-roadmap.md:22`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:12`

### Test Coverage
- `clients/bevy-ref/src/info_views.rs:738`
- `clients/bevy-ref/src/info_views.rs:759`
- `clients/bevy-ref/src/info_views.rs:773`

## Decision

TBD -- The architectural decision for FR-CIV-INFOVIEW-900 needs to be finalized based on implementation exploration.

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
