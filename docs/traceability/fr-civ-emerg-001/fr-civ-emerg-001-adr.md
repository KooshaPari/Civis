# ADR: FR-CIV-EMERG-001 -- Emergence mechanics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-EMERG-001
> Epic: FR-CIV-EMERG

## Context

FR-CIV-EMERG-001 is part of the FR-CIV-EMERG epic. This functional requirement captures: Emergence mechanics.

Implementing crate: `crates/emergence-oracle/src/`

### Referenced Source
- `crates/civ-emergence-metrics/src/dashboard.rs:1`
- `crates/civ-emergence-metrics/src/dashboard.rs:13`
- `crates/civ-emergence-metrics/src/dashboard.rs:14`
- `crates/civ-emergence-metrics/src/dashboard.rs:15`
- `crates/civ-emergence-metrics/src/dashboard.rs:16`
- `crates/civ-emergence-metrics/src/dashboard.rs:17`
- `crates/civ-emergence-metrics/src/dashboard.rs:28`
- `crates/engine/src/emergence_metrics.rs:113`

### Test Coverage
- `crates/civ-emergence-metrics/src/dashboard.rs:392`
- `crates/civ-emergence-metrics/src/dashboard.rs:425`
- `crates/engine/src/emergence_metrics.rs:579`

## Decision

TBD -- The architectural decision for FR-CIV-EMERG-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/emergence-oracle/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Emergence mechanics requirement in the simulation

### Negative
- Adds complexity to the emergence-oracle crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/emergence-oracle/`
2. **Option B**: Extract into a dedicated sub-crate
