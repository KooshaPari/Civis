# ADR: FR-SOC-HLT-004 -- Social health

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-SOC-HLT-004
> Epic: FR-SOC-HLT

## Context

FR-SOC-HLT-004 is part of the FR-SOC-HLT epic. This functional requirement captures: Social health.

Implementing crate: `crates/civ-institutions/src/`

### Referenced Source
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1675`
- `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md:1997`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-SOC-HLT-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-institutions/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Social health requirement in the simulation

### Negative
- Adds complexity to the civ-institutions crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-institutions/`
2. **Option B**: Extract into a dedicated sub-crate
