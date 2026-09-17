# ADR: FR-CIV-RES-001 -- Civ Res

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RES-001
> Epic: FR-CIV-RES

## Context

FR-CIV-RES-001 is part of the FR-CIV-RES epic. This functional requirement captures: Civ Res.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/reference/CODE_ENTITY_MAP.md:17`
- `docs/reference/FR_TRACKER.md:40`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-RES-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Civ Res requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
