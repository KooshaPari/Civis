# Plan: FR-CIV-SCALE-002 -- Scale and performance

> Date: 2026-09-17
> FR: FR-CIV-SCALE-002
> Epic: FR-CIV-SCALE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Scale and performance logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/scale_budget.rs:11`
- `crates/voxel/src/scale_budget.rs:277`
- `crates/voxel/src/scale_budget.rs:284`
- `crates/voxel/src/scale_budget.rs:300`
- `crates/voxel/src/scale_budget.rs:305`
- `crates/voxel/src/scale_budget.rs:306`
- `crates/voxel/src/scale_budget.rs:317`
- `crates/voxel/src/scale_budget.rs:367`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:970`
- `crates/voxel/src/scale_budget.rs:972`
- `crates/voxel/src/scale_budget.rs:985`
- `crates/voxel/src/scale_budget.rs:1006`
- `crates/voxel/src/scale_budget.rs:1025`

## Dependencies

- Epic: FR-CIV-SCALE
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
