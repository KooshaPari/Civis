# Plan: FR-CIV-UX-006 -- User experience

> Date: 2026-09-17
> FR: FR-CIV-UX-006
> Epic: FR-CIV-UX
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the User experience logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
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

## Dependencies

- Epic: FR-CIV-UX
- Implementing crate: `crates/hud/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p hud`
2. `cargo test -p hud`
3. `cargo clippy -p hud`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
