# ADR: FR-CIV-LEGENDS-GRAPH-01 -- Civ Legends Graph

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-GRAPH-01
> Epic: FR-CIV-LEGENDS-GRAPH

## Context

FR-CIV-LEGENDS-GRAPH-01 is part of the FR-CIV-LEGENDS-GRAPH epic. This functional requirement captures: Civ Legends Graph.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/legends/src/lib.rs:16`
- `docs/design/legends-engine.md:436`
- `docs/design/master-roadmap.md:23`

### Test Coverage
- `crates/legends/tests/saga_graph.rs:2`

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-GRAPH-01 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Legends Graph requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
