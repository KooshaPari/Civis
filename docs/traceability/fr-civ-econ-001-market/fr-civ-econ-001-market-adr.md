# ADR: FR-CIV-ECON-001-MARKET -- Economy and joule allocation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ECON-001-MARKET
> Epic: FR-CIV-ECON

## Context

FR-CIV-ECON-001-MARKET is part of the FR-CIV-ECON epic. This functional requirement captures: Economy and joule allocation.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/guides/COPILOT_L3_AGENTS.md:90`
- `docs/guides/COPILOT_L3_AGENTS.md:91`
- `docs/guides/COPILOT_L3_AGENTS.md:470`
- `docs/guides/COPILOT_L3_AGENTS.md:472`
- `docs/guides/COPILOT_L3_AGENTS.md:587`
- `docs/guides/COPILOT_L3_AGENTS.md:635`
- `docs/guides/TEST_FIRST_GUIDE.md:351`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-ECON-001-MARKET needs to be finalized based on implementation exploration.

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
