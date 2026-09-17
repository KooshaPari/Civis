# ADR: FR-CIV-MARKET-008 -- Market dynamics

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-MARKET-008
> Epic: FR-CIV-MARKET

## Context

FR-CIV-MARKET-008 is part of the FR-CIV-MARKET epic. This functional requirement captures: Market dynamics.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/design/polities-markets.md:146`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-MARKET-008 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Market dynamics requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
