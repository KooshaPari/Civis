# ADR: FR-CIV-TRAFFIC-LANE-002 -- Traffic lanes

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TRAFFIC-LANE-002
> Epic: FR-CIV-TRAFFIC-LANE

## Context

FR-CIV-TRAFFIC-LANE-002 is part of the FR-CIV-TRAFFIC-LANE epic. This functional requirement captures: Traffic lanes.

Implementing crate: `crates/civ-traffic/src/`

### Referenced Source
- `crates/civ-traffic/src/lane.rs:4`

### Test Coverage
- `crates/civ-traffic/src/lane.rs:346`

## Decision

TBD -- The architectural decision for FR-CIV-TRAFFIC-LANE-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civ-traffic/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Traffic lanes requirement in the simulation

### Negative
- Adds complexity to the civ-traffic crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civ-traffic/`
2. **Option B**: Extract into a dedicated sub-crate
