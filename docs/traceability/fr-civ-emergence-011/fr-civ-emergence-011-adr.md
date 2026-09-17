# ADR: FR-CIV-EMERGENCE-011 -- Emergence metrics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-EMERGENCE-011
> Epic: FR-CIV-EMERGENCE

## Context

FR-CIV-EMERGENCE-011 is part of the FR-CIV-EMERGENCE epic. This functional requirement captures: Emergence metrics.

Implementing crate: `crates/emergence-oracle/src/`

### Referenced Source
- `docs/guides/voxel-emergent-vision-and-migration.md:98`
- `docs/guides/voxel-emergent-vision-and-migration.md:144`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-EMERGENCE-011 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/emergence-oracle/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Emergence metrics requirement in the simulation

### Negative
- Adds complexity to the emergence-oracle crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/emergence-oracle/`
2. **Option B**: Extract into a dedicated sub-crate
