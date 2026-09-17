# ADR: FR-CIV-PLANET-060 -- Planetary generation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PLANET-060
> Epic: FR-CIV-PLANET

## Context

FR-CIV-PLANET-060 is part of the FR-CIV-PLANET epic. This functional requirement captures: Planetary generation.

Implementing crate: `crates/planet/src/`

### Referenced Source
- `crates/engine/src/hash_chain.rs:5`
- `crates/engine/src/hash_chain.rs:129`
- `crates/engine/src/replay.rs:42`
- `crates/engine/src/replay.rs:254`

### Test Coverage
- `crates/engine/src/engine.rs:3014`
- `crates/engine/src/hash_chain.rs:235`

## Decision

TBD -- The architectural decision for FR-CIV-PLANET-060 needs to be finalized based on implementation exploration.

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
