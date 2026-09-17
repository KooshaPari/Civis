# ADR: FR-CIV-DIPLO-008 -- Diplomacy and treaties

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-DIPLO-008
> Epic: FR-CIV-DIPLO

## Context

FR-CIV-DIPLO-008 is part of the FR-CIV-DIPLO epic. This functional requirement captures: Diplomacy and treaties.

Implementing crate: `crates/diplomacy/src/`

### Referenced Source
- `crates/diplomacy/src/lib.rs:277`
- `crates/diplomacy/src/lib.rs:437`
- `crates/diplomacy/src/lib.rs:451`
- `crates/diplomacy/src/lib.rs:539`

### Test Coverage
- `crates/diplomacy/src/lib.rs:1340`
- `crates/diplomacy/src/lib.rs:1342`
- `crates/diplomacy/src/lib.rs:1351`
- `crates/diplomacy/src/lib.rs:1380`
- `crates/diplomacy/src/lib.rs:1406`
- `crates/diplomacy/src/lib.rs:1439`
- `crates/diplomacy/src/lib.rs:1474`
- `crates/diplomacy/src/lib.rs:1512`

## Decision

TBD -- The architectural decision for FR-CIV-DIPLO-008 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/diplomacy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Diplomacy and treaties requirement in the simulation

### Negative
- Adds complexity to the diplomacy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/diplomacy/`
2. **Option B**: Extract into a dedicated sub-crate
