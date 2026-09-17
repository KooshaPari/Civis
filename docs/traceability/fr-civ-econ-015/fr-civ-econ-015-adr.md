# ADR: FR-CIV-ECON-015 -- Economy and joule allocation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ECON-015
> Epic: FR-CIV-ECON

## Context

FR-CIV-ECON-015 is part of the FR-CIV-ECON epic. This functional requirement captures: Economy and joule allocation.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `crates/economy/src/chains.rs:2`

### Test Coverage
- `crates/economy/src/chains.rs:391`
- `crates/economy/src/chains.rs:414`
- `crates/economy/src/chains.rs:444`
- `crates/economy/src/chains.rs:466`
- `crates/economy/src/chains.rs:494`
- `crates/economy/src/chains.rs:537`
- `crates/economy/src/chains.rs:577`
- `crates/economy/src/chains.rs:598`

## Decision

TBD -- The architectural decision for FR-CIV-ECON-015 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/economy/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Economy and joule allocation requirement in the simulation

### Negative
- Adds complexity to the economy crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/economy/`
2. **Option B**: Extract into a dedicated sub-crate
