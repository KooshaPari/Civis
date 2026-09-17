# ADR: FR-CIV-INSPECT-920 -- Inspection tools

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INSPECT-920
> Epic: FR-CIV-INSPECT

## Context

FR-CIV-INSPECT-920 is part of the FR-CIV-INSPECT epic. This functional requirement captures: Inspection tools.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/agileplus/epics/civ-w4-perception.md:22`
- `docs/agileplus/epics/civ-w4-perception.md:37`
- `docs/agileplus/README.md:23`
- `docs/design/onboarding-qol.md:138`
- `docs/specs/requirements/FR-CIV-INSPECT.md:16`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-INSPECT-920 needs to be finalized based on implementation exploration.

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
