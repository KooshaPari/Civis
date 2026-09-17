# ADR: FR-CIV-TACTICS-042 -- Tactics and strategy

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-TACTICS-042
> Epic: FR-CIV-TACTICS

## Context

FR-CIV-TACTICS-042 is part of the FR-CIV-TACTICS epic. This functional requirement captures: Tactics and strategy.

Implementing crate: `crates/tactics/src/`

### Referenced Source
- `crates/tactics/src/fog_of_war.rs:1`
- `crates/tactics/src/war_bridge.rs:59`
- `docs/development-guide/p-w1-kickoff.md:44`

### Test Coverage
- `crates/tactics/src/war_bridge.rs:425`
- `crates/tactics/src/war_bridge.rs:426`

## Decision

TBD -- The architectural decision for FR-CIV-TACTICS-042 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/tactics/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Tactics and strategy requirement in the simulation

### Negative
- Adds complexity to the tactics crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/tactics/`
2. **Option B**: Extract into a dedicated sub-crate
