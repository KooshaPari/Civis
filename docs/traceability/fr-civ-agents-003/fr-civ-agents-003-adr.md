# ADR: FR-CIV-AGENTS-003 -- Agent systems

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-AGENTS-003
> Epic: FR-CIV-AGENTS

## Context

FR-CIV-AGENTS-003 is part of the FR-CIV-AGENTS epic. This functional requirement captures: Agent systems.

Implementing crate: `crates/ai/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
- `crates/agents/src/lib.rs:772`

## Decision

TBD -- The architectural decision for FR-CIV-AGENTS-003 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/ai/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Agent systems requirement in the simulation

### Negative
- Adds complexity to the ai crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/ai/`
2. **Option B**: Extract into a dedicated sub-crate
