# ADR: FR-CIV-3D-009 -- Core civilisation simulation

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-3D-009
> Epic: FR-CIV

## Context

FR-CIV-3D-009 is part of the FR-CIV epic. This functional requirement captures: Core civilisation simulation.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/specs/CIV-0601-3d-asset-transition-and-agentic-gen-spec.md:1952`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-3D-009 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the Core civilisation simulation requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
