# ADR: FR-CIV-CORE-001 -- Core simulation engine

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-CORE-001
> Epic: FR-CIV-CORE

## Context

FR-CIV-CORE-001 is part of the FR-CIV-CORE epic. This functional requirement captures: Core simulation engine.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/AGILE_WORKSTREAM.md:372`
- `docs/AGILE_WORKSTREAM.md:444`
- `docs/AGILE_WORKSTREAM.md:455`
- `docs/AGILE_WORKSTREAM.md:512`
- `docs/AGILE_WORKSTREAM.md:586`
- `docs/AGILE_WORKSTREAM.md:590`
- `docs/AGILE_WORKSTREAM.md:618`
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2104`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:692`
- `crates/build/tests/fr_matrix_batch12.rs:695`

## Decision

TBD -- The architectural decision for FR-CIV-CORE-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core simulation engine requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
