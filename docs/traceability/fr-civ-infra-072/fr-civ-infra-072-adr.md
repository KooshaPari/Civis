# ADR: FR-CIV-INFRA-072 -- Infrastructure systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-INFRA-072
> Epic: FR-CIV-INFRA

## Context

FR-CIV-INFRA-072 is part of the FR-CIV-INFRA epic. This functional requirement captures: Infrastructure systems.

Implementing crate: `crates/infra/src/`

### Referenced Source
- `crates/civ-traffic/src/grid.rs:18`

### Test Coverage
- `crates/civ-traffic/src/grid.rs:356`
- `crates/civ-traffic/src/grid.rs:358`
- `crates/civ-traffic/src/grid.rs:381`
- `crates/civ-traffic/src/grid.rs:400`

## Decision

TBD -- The architectural decision for FR-CIV-INFRA-072 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/infra/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Infrastructure systems requirement in the simulation

### Negative
- Adds complexity to the infra crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/infra/`
2. **Option B**: Extract into a dedicated sub-crate
