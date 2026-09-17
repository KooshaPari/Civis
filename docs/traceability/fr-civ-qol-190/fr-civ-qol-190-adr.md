# ADR: FR-CIV-QOL-190 -- Civ Qol

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-QOL-190
> Epic: FR-CIV-QOL

## Context

FR-CIV-QOL-190 is part of the FR-CIV-QOL epic. This functional requirement captures: Civ Qol.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `docs/design/onboarding-qol.md:191`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-QOL-190 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Qol requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
