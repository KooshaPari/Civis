# ADR: FR-CIV-ECON-001 -- Economy and joule allocation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-ECON-001
> Epic: FR-CIV-ECON

## Context

FR-CIV-ECON-001 is part of the FR-CIV-ECON epic. This functional requirement captures: Economy and joule allocation.

Implementing crate: `crates/economy/src/`

### Referenced Source
- `docs/AGILE_WORKSTREAM.md:65`
- `docs/AGILE_WORKSTREAM.md:106`
- `docs/AGILE_WORKSTREAM.md:168`
- `docs/AGILE_WORKSTREAM.md:170`
- `docs/AGILE_WORKSTREAM.md:177`
- `docs/AGILE_WORKSTREAM.md:378`
- `docs/guides/COPILOT_L3_AGENTS.md:39`
- `docs/guides/COPILOT_L3_AGENTS.md:47`

### Test Coverage
- `crates/economy/src/chains.rs:627`
- `crates/economy/src/chains.rs:629`

## Decision

TBD -- The architectural decision for FR-CIV-ECON-001 needs to be finalized based on implementation exploration.

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
