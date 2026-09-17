# ADR: FR-CIV-RTS-RENDER-005 -- RTS rendering

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RTS-RENDER-005
> Epic: FR-CIV-RTS-RENDER

## Context

FR-CIV-RTS-RENDER-005 is part of the FR-CIV-RTS-RENDER epic. This functional requirement captures: RTS rendering.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:3211`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-RTS-RENDER-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the RTS rendering requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
