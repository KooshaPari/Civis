# ADR: FR-CIV-ACT-001 -- Actor lifecycle

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ACT-001
> Epic: FR-CIV-ACT

## Context

FR-CIV-ACT-001 is part of the FR-CIV-ACT epic. This functional requirement captures: Actor lifecycle.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2103`
- `docs/reference/FR_TRACKER.md:28`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:179`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:509`
- `docs/reports/STATUS_REPORT.md:97`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:115`
- `crates/build/tests/fr_matrix_batch12.rs:118`

## Decision

TBD -- The architectural decision for FR-CIV-ACT-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Actor lifecycle requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
