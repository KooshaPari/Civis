# ADR: FR-CIV-UX-000 -- User experience

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-UX-000
> Epic: FR-CIV-UX

## Context

FR-CIV-UX-000 is part of the FR-CIV-UX epic. This functional requirement captures: User experience.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/godot-ref/rust/src/ux.rs:18`
- `docs/development-guide/fr-3d-additions.md:109`
- `docs/development-guide/fr-p-u1-roadmap.md:10`
- `docs/roadmap/product-quality-ladder.md:16`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-UX-000 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the User experience requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
