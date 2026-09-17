# ADR: FR-CIV-GOV-001 -- Government and governance

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-GOV-001
> Epic: FR-CIV-GOV

## Context

FR-CIV-GOV-001 is part of the FR-CIV-GOV epic. This functional requirement captures: Government and governance.

Implementing crate: `crates/civ-institutions/src/`

### Referenced Source
- `crates/diplomacy/src/lib.rs:18`
- `docs/reference/agileplus-artifacts-index.md:137`
- `docs/reference/agileplus-artifacts-index.md:285`

### Test Coverage
- `crates/diplomacy/src/lib.rs:990`

## Decision

TBD -- The architectural decision for FR-CIV-GOV-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-institutions/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Government and governance requirement in the simulation

### Negative
- Adds complexity to the civ-institutions crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-institutions/`
2. **Option B**: Extract into a dedicated sub-crate
