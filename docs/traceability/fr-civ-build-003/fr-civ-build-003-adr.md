# ADR: FR-CIV-BUILD-003 -- Building tiers and construction

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BUILD-003
> Epic: FR-CIV-BUILD

## Context

FR-CIV-BUILD-003 is part of the FR-CIV-BUILD epic. This functional requirement captures: Building tiers and construction.

Implementing crate: `crates/physics-substrate/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:274`

### Test Coverage
- `crates/build/src/lib.rs:543`
- `crates/build/tests/fr_matrix_batch12.rs:510`
- `crates/build/tests/fr_matrix_batch12.rs:513`

## Decision

TBD -- The architectural decision for FR-CIV-BUILD-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/physics-substrate/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Building tiers and construction requirement in the simulation

### Negative
- Adds complexity to the physics-substrate crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/physics-substrate/`
2. **Option B**: Extract into a dedicated sub-crate
