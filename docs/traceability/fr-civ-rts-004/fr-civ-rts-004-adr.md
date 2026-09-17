# ADR: FR-CIV-RTS-004 -- RTS gameplay

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-RTS-004
> Epic: FR-CIV-RTS

## Context

FR-CIV-RTS-004 is part of the FR-CIV-RTS epic. This functional requirement captures: RTS gameplay.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/specs/CIV-0300-rts-ui-ux-spec.md:1143`
- `docs/specs/CIV-0300-rts-ui-ux-spec.md:1317`
- `docs/specs/CIV-0300-rts-ui-ux-spec.md:2007`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-RTS-004 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the RTS gameplay requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
