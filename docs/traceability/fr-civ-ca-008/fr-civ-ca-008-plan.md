# Plan: FR-CIV-CA-008 -- Combat and military

> Date: 2026-09-17
> FR: FR-CIV-CA-008
> Epic: FR-CIV-CA
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/ai/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Combat and military logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/fluid_ca.rs:33`
- `crates/voxel/src/fluid_ca.rs:60`
- `crates/voxel/src/fluid_ca.rs:85`
- `crates/voxel/src/fluid_ca.rs:248`
- `crates/voxel/src/fluid_ca.rs:257`
- `crates/voxel/src/fluid_ca.rs:265`
- `crates/voxel/src/fluid_ca.rs:694`
- `crates/voxel/src/fluid_ca.rs:1076`

### Test Coverage
- `crates/voxel/src/fluid_ca.rs:2000`
- `crates/voxel/src/fluid_ca.rs:2005`
- `crates/voxel/src/fluid_ca.rs:2032`
- `crates/voxel/src/fluid_ca.rs:2033`

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
