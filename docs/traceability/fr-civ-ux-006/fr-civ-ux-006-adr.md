# ADR: FR-CIV-UX-006 -- User experience

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-UX-006
> Epic: FR-CIV-UX

## Context

FR-CIV-UX-006 is part of the FR-CIV-UX epic. This functional requirement captures: User experience.

Implementing crate: `crates/hud/src/`

### Referenced Source
- `clients/godot-ref/rust/src/ux.rs:52`
- `clients/unreal-show/Intermediate/Build/Win64/UnrealEditor/Inc/CivShow/UHT/CivProtocolClient.gen.cpp:427`
- `clients/unreal-show/Intermediate/Build/Win64/UnrealEditor/Inc/CivShow/UHT/CivProtocolClient.gen.cpp:431`
- `clients/unreal-show/Source/CivShow/CivProtocolClient.h:32`
- `crates/engine/src/spawn.rs:1`
- `crates/server/src/jsonrpc.rs:58`
- `crates/server/src/jsonrpc.rs:868`
- `crates/server/src/jsonrpc.rs:1381`

### Test Coverage
- `crates/server/tests/ws_smoke.rs:1715`
- `crates/server/tests/ws_smoke.rs:1716`

## Decision

TBD -- The architectural decision for FR-CIV-UX-006 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/hud/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the User experience requirement in the simulation

### Negative
- Adds complexity to the hud crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/hud/`
2. **Option B**: Extract into a dedicated sub-crate
