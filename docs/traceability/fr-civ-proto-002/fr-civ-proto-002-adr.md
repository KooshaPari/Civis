# ADR: FR-CIV-PROTO-002 -- 3D protocol

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-PROTO-002
> Epic: FR-CIV-PROTO

## Context

FR-CIV-PROTO-002 is part of the FR-CIV-PROTO epic. This functional requirement captures: 3D protocol.

Implementing crate: `crates/protocol-3d/src/`

### Referenced Source
- `docs/AGILE_WORKSTREAM.md:266`
- `docs/AGILE_WORKSTREAM.md:296`
- `docs/AGILE_WORKSTREAM.md:301`
- `docs/AGILE_WORKSTREAM.md:310`
- `docs/specs/CIV-0200-client-protocol.md:1129`

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-PROTO-002 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/protocol-3d/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the 3D protocol requirement in the simulation

### Negative
- Adds complexity to the protocol-3d crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/protocol-3d/`
2. **Option B**: Extract into a dedicated sub-crate
