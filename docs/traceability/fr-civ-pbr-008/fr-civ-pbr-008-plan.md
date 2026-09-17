# Plan: FR-CIV-PBR-008 -- Physically based rendering

> Date: 2026-09-17
> FR: FR-CIV-PBR-008
> Epic: FR-CIV-PBR
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Physically based rendering logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/material_pbr.rs:21`
- `crates/voxel/src/material_pbr.rs:394`

### Test Coverage
- `crates/voxel/src/material_pbr.rs:1082`
- `crates/voxel/src/material_pbr.rs:1084`
- `crates/voxel/src/material_pbr.rs:1098`
- `crates/voxel/src/material_pbr.rs:1112`

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
