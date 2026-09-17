# ADR: FR-CIV-LIFE-003 -- Life simulation and needs

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LIFE-003
> Epic: FR-CIV-LIFE

## Context

FR-CIV-LIFE-003 is part of the FR-CIV-LIFE epic. This functional requirement captures: Life simulation and needs.

Implementing crate: `crates/species/src/`

### Referenced Source
- `crates/engine/src/engine.rs:452`
- `crates/engine/src/engine.rs:2261`
- `crates/needs/src/lib.rs:15`

### Test Coverage
- `crates/engine/src/engine.rs:2458`
- `crates/needs/src/lib.rs:428`
- `crates/needs/src/lib.rs:455`
- `crates/needs/src/lib.rs:588`
- `crates/needs/src/lib.rs:679`
- `crates/needs/src/lib.rs:895`
- `crates/needs/src/lib.rs:930`
- `crates/needs/src/lib.rs:968`

## Decision

TBD -- The architectural decision for FR-CIV-LIFE-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/species/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Life simulation and needs requirement in the simulation

### Negative
- Adds complexity to the species crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/species/`
2. **Option B**: Extract into a dedicated sub-crate
