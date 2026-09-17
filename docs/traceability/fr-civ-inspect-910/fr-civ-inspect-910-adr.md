# ADR: FR-CIV-INSPECT-910 -- Inspection tools

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INSPECT-910
> Epic: FR-CIV-INSPECT

## Context

FR-CIV-INSPECT-910 is part of the FR-CIV-INSPECT epic. This functional requirement captures: Inspection tools.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/bevy-ref/src/inspect.rs:13`
- `clients/bevy-ref/src/inspect.rs:104`
- `clients/bevy-ref/src/inspect.rs:323`
- `docs/agileplus/epics/civ-w4-perception.md:21`
- `docs/agileplus/epics/civ-w4-perception.md:37`
- `docs/agileplus/README.md:23`
- `docs/specs/requirements/FR-CIV-INSPECT.md:15`

### Test Coverage
- `clients/bevy-ref/src/inspect.rs:392`

## Decision

TBD -- The architectural decision for FR-CIV-INSPECT-910 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Inspection tools requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
