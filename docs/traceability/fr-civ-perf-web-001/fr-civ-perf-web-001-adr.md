# ADR: FR-CIV-PERF-WEB-001 -- Civ Perf Web

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PERF-WEB-001
> Epic: FR-CIV-PERF-WEB

## Context

FR-CIV-PERF-WEB-001 is part of the FR-CIV-PERF-WEB epic. This functional requirement captures: Civ Perf Web.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3219`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-PERF-WEB-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Perf Web requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
