# ADR: FR-CIV-ECON-002-JOULE -- Economy and joule allocation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ECON-002-JOULE
> Epic: FR-CIV-ECON

## Context

FR-CIV-ECON-002-JOULE is part of the FR-CIV-ECON epic. This functional requirement captures: Economy and joule allocation.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/guides/COPILOT_L3_AGENTS.md:92`
- `docs/guides/COPILOT_L3_AGENTS.md:93`
- `docs/guides/COPILOT_L3_AGENTS.md:474`
- `docs/guides/COPILOT_L3_AGENTS.md:476`

### Test Coverage
- `crates/economy/src/allocator.rs:971`
- `crates/economy/src/allocator.rs:1026`
- `crates/economy/tests/allocation_behavior.rs:311`

## Decision

TBD -- The architectural decision for FR-CIV-ECON-002-JOULE needs to be finalized based on implementation exploration.

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
