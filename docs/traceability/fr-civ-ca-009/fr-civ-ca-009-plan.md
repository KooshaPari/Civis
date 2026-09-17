# Plan: FR-CIV-CA-009 -- Combat and military

> Date: 2026-09-17
> FR: FR-CIV-CA-009
> Epic: FR-CIV-CA
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/ai/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Combat and military logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/engine.rs:417`
- `crates/engine/src/engine.rs:1449`
- `crates/engine/src/engine.rs:1501`
- `crates/voxel/src/fluid_ca.rs:124`
- `crates/voxel/src/fluid_ca.rs:141`

### Test Coverage
- `crates/engine/src/engine.rs:3343`
- `crates/engine/src/engine.rs:3346`
- `crates/engine/src/engine.rs:3356`
- `crates/voxel/src/fluid_ca.rs:2105`
- `crates/voxel/src/fluid_ca.rs:2110`

## Dependencies

- Epic: FR-CIV-CA
- Implementing crate: `crates/ai/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p ai`
2. `cargo test -p ai`
3. `cargo clippy -p ai`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
