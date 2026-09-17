# ADR: FR-SAVE-003 -- Save system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SAVE-003
> Epic: FR-SAVE

## Context

FR-SAVE-003 is part of the FR-SAVE epic. This functional requirement captures: Save system.

Implementing crate: `crates/save-db/src/`

### Referenced Source
- `docs/specs/CIV-1000-save-load-persistence-spec.md:2802`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-SAVE-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/save-db/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Save system requirement in the simulation

### Negative
- Adds complexity to the save-db crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/save-db/`
2. **Option B**: Extract into a dedicated sub-crate
