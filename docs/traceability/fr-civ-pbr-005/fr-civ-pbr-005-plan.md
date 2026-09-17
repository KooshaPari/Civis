# Plan: FR-CIV-PBR-005 -- Physically based rendering

> Date: 2026-09-17
> FR: FR-CIV-PBR-005
> Epic: FR-CIV-PBR
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Physically based rendering logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/material_pbr.rs:17`
- `crates/voxel/src/material_pbr.rs:144`

### Test Coverage
- `crates/voxel/src/material_pbr.rs:971`
- `crates/voxel/src/material_pbr.rs:973`
- `crates/voxel/src/material_pbr.rs:995`

## Dependencies

- Epic: FR-CIV-PBR
- Implementing crate: `crates/engine/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p engine`
2. `cargo test -p engine`
3. `cargo clippy -p engine`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
