# ADR: FR-CIV-BUILD-001 -- Building tiers and construction

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BUILD-001
> Epic: FR-CIV-BUILD

## Context

FR-CIV-BUILD-001 is part of the FR-CIV-BUILD epic. This functional requirement captures: Building tiers and construction.

Implementing crate: `crates/physics-substrate/src/`

### Referenced Source
- `docs/development-guide/fr-3d-additions.md:30`
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:272`

### Test Coverage
- `crates/build/src/lib.rs:515`
- `crates/build/src/lib.rs:726`
- `crates/build/tests/fr_matrix_batch12.rs:475`
- `crates/build/tests/fr_matrix_batch12.rs:478`

## Decision

TBD -- The architectural decision for FR-CIV-BUILD-001 needs to be finalized based on implementation exploration.

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
