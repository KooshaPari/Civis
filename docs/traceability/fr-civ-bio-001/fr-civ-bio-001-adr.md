# ADR: FR-CIV-BIO-001 -- Biological simulation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-BIO-001
> Epic: FR-CIV-BIO

## Context

FR-CIV-BIO-001 is part of the FR-CIV-BIO epic. This functional requirement captures: Biological simulation.

Implementing crate: `crates/species/src/`

### Referenced Source
- `docs/guides/voxel-emergent-vision-and-migration.md:33`
- `docs/guides/voxel-emergent-vision-and-migration.md:45`
- `docs/guides/voxel-emergent-vision-and-migration.md:62`
- `docs/guides/voxel-emergent-vision-and-migration.md:79`
- `docs/guides/voxel-emergent-vision-and-migration.md:97`
- `docs/guides/voxel-emergent-vision-and-migration.md:205`
- `docs/guides/voxel-emergent-vision-and-migration.md:207`
- `docs/reference/agileplus-artifacts-index.md:153`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:381`
- `crates/build/tests/fr_matrix_batch12.rs:384`

## Decision

TBD -- The architectural decision for FR-CIV-BIO-001 needs to be finalized based on implementation exploration.

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
