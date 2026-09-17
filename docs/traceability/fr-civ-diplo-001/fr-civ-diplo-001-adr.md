# ADR: FR-CIV-DIPLO-001 -- Diplomacy and treaties

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-DIPLO-001
> Epic: FR-CIV-DIPLO

## Context

FR-CIV-DIPLO-001 is part of the FR-CIV-DIPLO epic. This functional requirement captures: Diplomacy and treaties.

Implementing crate: `crates/diplomacy/src/`

### Referenced Source
- `crates/diplomacy/Cargo.toml:3`
- `crates/diplomacy/src/lib.rs:1`
- `crates/diplomacy/src/lib.rs:15`
- `crates/diplomacy/src/lib.rs:437`
- `crates/diplomacy/src/lib.rs:439`
- `crates/diplomacy/src/lib.rs:522`
- `crates/diplomacy/src/lib.rs:536`
- `docs/reference/agileplus-artifacts-index.md:137`

### Test Coverage
- `crates/diplomacy/src/lib.rs:884`
- `crates/diplomacy/src/lib.rs:942`
- `crates/diplomacy/src/lib.rs:1161`
- `crates/diplomacy/src/lib.rs:1212`
- `crates/diplomacy/src/lib.rs:1238`
- `crates/diplomacy/src/lib.rs:1346`

## Decision

TBD -- The architectural decision for FR-CIV-DIPLO-001 needs to be finalized based on implementation exploration.

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
