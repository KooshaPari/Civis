# ADR: FR-CORE-001 -- Core system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CORE-001
> Epic: FR-CORE

## Context

FR-CORE-001 is part of the FR-CORE epic. This functional requirement captures: Core system.

Implementing crate: `crates/engine/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:41`
- `docs/reference/agileplus-artifacts-index.md:253`
- `README.md:202`

### Test Coverage
- `crates/engine/src/engine.rs:2304`
- `crates/engine/tests/tick_budget.rs:1`
- `crates/engine/tests/tick_budget.rs:16`
- `crates/engine/tests/tick_budget.rs:17`

## Decision

TBD -- The architectural decision for FR-CORE-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/engine/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core system requirement in the simulation

### Negative
- Adds complexity to the engine crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/engine/`
2. **Option B**: Extract into a dedicated sub-crate
