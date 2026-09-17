# ADR: FR-CLIENT-001 -- Client system

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CLIENT-001
> Epic: FR-CLIENT

## Context

FR-CLIENT-001 is part of the FR-CLIENT epic. This functional requirement captures: Client system.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/reference/agileplus-artifacts-index.md:201`
- `docs/reference/agileplus-artifacts-index.md:299`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CLIENT-001 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Client system requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
