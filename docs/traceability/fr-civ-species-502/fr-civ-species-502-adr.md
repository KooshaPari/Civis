# ADR: FR-CIV-SPECIES-502 -- Species definitions

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-SPECIES-502
> Epic: FR-CIV-SPECIES

## Context

FR-CIV-SPECIES-502 is part of the FR-CIV-SPECIES epic. This functional requirement captures: Species definitions.

Implementing crate: `crates/species/src/`

### Referenced Source
- `docs/design/species-sentience.md:193`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-SPECIES-502 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/species/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Species definitions requirement in the simulation

### Negative
- Adds complexity to the species crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/species/`
2. **Option B**: Extract into a dedicated sub-crate
