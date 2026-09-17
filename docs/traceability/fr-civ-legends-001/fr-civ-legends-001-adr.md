# ADR: FR-CIV-LEGENDS-001 -- Legend and narrative system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-001
> Epic: FR-CIV-LEGENDS

## Context

FR-CIV-LEGENDS-001 is part of the FR-CIV-LEGENDS epic. This functional requirement captures: Legend and narrative system.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/legends/src/model.rs:240`

### Test Coverage
- `crates/legends/tests/fr_legends_completion.rs:1`
- `crates/legends/tests/fr_legends_completion.rs:5`
- `crates/legends/tests/fr_legends_completion.rs:31`
- `crates/legends/tests/fr_legends_completion.rs:34`
- `crates/legends/tests/fr_legends_completion.rs:60`
- `crates/legends/tests/fr_legends_completion.rs:85`

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Legend and narrative system requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
