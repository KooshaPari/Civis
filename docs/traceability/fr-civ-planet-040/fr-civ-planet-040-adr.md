# ADR: FR-CIV-PLANET-040 -- Planetary generation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PLANET-040
> Epic: FR-CIV-PLANET

## Context

FR-CIV-PLANET-040 is part of the FR-CIV-PLANET epic. This functional requirement captures: Planetary generation.

Implementing crate: `crates/planet/src/`

### Referenced Source
- `crates/engine/src/engine.rs:2255`
- `crates/planet/src/geology.rs:1`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:1`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:34`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:213`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:249`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:307`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:337`

### Test Coverage
- `crates/engine/src/engine.rs:3293`
- `crates/planet/src/geology.rs:106`
- `docs/superpowers/plans/2026-05-28-fr-civ-planet-040-geology-seed.md:107`

## Decision

TBD -- The architectural decision for FR-CIV-PLANET-040 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/planet/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Planetary generation requirement in the simulation

### Negative
- Adds complexity to the planet crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/planet/`
2. **Option B**: Extract into a dedicated sub-crate
