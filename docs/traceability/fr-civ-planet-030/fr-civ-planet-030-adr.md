# ADR: FR-CIV-PLANET-030 -- Planetary generation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PLANET-030
> Epic: FR-CIV-PLANET

## Context

FR-CIV-PLANET-030 is part of the FR-CIV-PLANET epic. This functional requirement captures: Planetary generation.

Implementing crate: `crates/planet/src/`

### Referenced Source
- `crates/engine/src/engine.rs:442`
- `crates/engine/src/engine.rs:1284`
- `crates/engine/src/engine.rs:2250`
- `crates/planet/src/weather.rs:1`

### Test Coverage
- `crates/engine/src/engine.rs:3292`
- `crates/engine/src/engine.rs:3294`

## Decision

TBD -- The architectural decision for FR-CIV-PLANET-030 needs to be finalized based on implementation exploration.

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
