# ADR: FR-STOR-001 -- Storage

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-STOR-001
> Epic: FR-STOR

## Context

FR-STOR-001 is part of the FR-STOR epic. This functional requirement captures: Storage.

Implementing crate: `crates/save-db/src/`

### Referenced Source
- `docs/models/civ-sim/OPS_GOVERNANCE_SPEC.md:1931`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-STOR-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/save-db/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Storage requirement in the simulation

### Negative
- Adds complexity to the save-db crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/save-db/`
2. **Option B**: Extract into a dedicated sub-crate
