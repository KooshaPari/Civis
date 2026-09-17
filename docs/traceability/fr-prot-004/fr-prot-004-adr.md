# ADR: FR-PROT-004 -- Protocol

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-PROT-004
> Epic: FR-PROT

## Context

FR-PROT-004 is part of the FR-PROT epic. This functional requirement captures: Protocol.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-PROT-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Protocol requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
