# ADR: FR-CIV-LEGENDS-INGEST-02 -- Civ Legends Ingest

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-LEGENDS-INGEST-02
> Epic: FR-CIV-LEGENDS-INGEST

## Context

FR-CIV-LEGENDS-INGEST-02 is part of the FR-CIV-LEGENDS-INGEST epic. This functional requirement captures: Civ Legends Ingest.

Implementing crate: `crates/legends/src/`

### Referenced Source
- `crates/legends/src/worker.rs:1`
- `docs/design/legends-engine.md:437`

### Test Coverage
- `crates/engine/src/emergence.rs:798`

## Decision

TBD -- The architectural decision for FR-CIV-LEGENDS-INGEST-02 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/legends/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Legends Ingest requirement in the simulation

### Negative
- Adds complexity to the legends crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/legends/`
2. **Option B**: Extract into a dedicated sub-crate
