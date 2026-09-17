# ADR: FR-CIV-DIFFUSION-014 -- Cultural diffusion

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-DIFFUSION-014
> Epic: FR-CIV-DIFFUSION

## Context

FR-CIV-DIFFUSION-014 is part of the FR-CIV-DIFFUSION epic. This functional requirement captures: Cultural diffusion.

Implementing crate: `crates/diffusion/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/diffusion/src/lib.rs:308`

## Decision

TBD -- The architectural decision for FR-CIV-DIFFUSION-014 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/diffusion/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Cultural diffusion requirement in the simulation

### Negative
- Adds complexity to the diffusion crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/diffusion/`
2. **Option B**: Extract into a dedicated sub-crate
