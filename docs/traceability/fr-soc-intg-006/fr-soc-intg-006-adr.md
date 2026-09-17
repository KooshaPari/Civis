# ADR: FR-SOC-INTG-006 -- Social integration

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SOC-INTG-006
> Epic: FR-SOC-INTG

## Context

FR-SOC-INTG-006 is part of the FR-SOC-INTG epic. This functional requirement captures: Social integration.

Implementing crate: `crates/civ-institutions/src/`

### Referenced Source
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4517`
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:4579`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-SOC-INTG-006 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-institutions/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Social integration requirement in the simulation

### Negative
- Adds complexity to the civ-institutions crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-institutions/`
2. **Option B**: Extract into a dedicated sub-crate
