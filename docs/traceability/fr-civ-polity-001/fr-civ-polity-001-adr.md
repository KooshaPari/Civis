# ADR: FR-CIV-POLITY-001 -- Polity system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-POLITY-001
> Epic: FR-CIV-POLITY

## Context

FR-CIV-POLITY-001 is part of the FR-CIV-POLITY epic. This functional requirement captures: Polity system.

Implementing crate: `crates/diplomacy/src/`

### Referenced Source
- `docs/design/master-roadmap.md:25`
- `docs/design/polities-markets.md:37`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-POLITY-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/diplomacy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Polity system requirement in the simulation

### Negative
- Adds complexity to the diplomacy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/diplomacy/`
2. **Option B**: Extract into a dedicated sub-crate
