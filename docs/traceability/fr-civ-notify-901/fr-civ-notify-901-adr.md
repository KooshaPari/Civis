# ADR: FR-CIV-NOTIFY-901 -- Notification system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-NOTIFY-901
> Epic: FR-CIV-NOTIFY

## Context

FR-CIV-NOTIFY-901 is part of the FR-CIV-NOTIFY epic. This functional requirement captures: Notification system.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w6-ui.md:13`
- `docs/agileplus/epics/civ-w6-ui.md:26`
- `docs/agileplus/README.md:25`
- `docs/design/onboarding-qol.md:212`
- `docs/specs/requirements/FR-CIV-NOTIFY.md:12`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-NOTIFY-901 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Notification system requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
