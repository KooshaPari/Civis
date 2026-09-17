# ADR: FR-CIV-BIO-003 -- Biological simulation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BIO-003
> Epic: FR-CIV-BIO

## Context

FR-CIV-BIO-003 is part of the FR-CIV-BIO epic. This functional requirement captures: Biological simulation.

Implementing crate: `crates/species/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:153`
- `docs/reference/agileplus-artifacts-index.md:289`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:434`
- `crates/build/tests/fr_matrix_batch12.rs:437`

## Decision

TBD -- The architectural decision for FR-CIV-BIO-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/species/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Biological simulation requirement in the simulation

### Negative
- Adds complexity to the species crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/species/`
2. **Option B**: Extract into a dedicated sub-crate
