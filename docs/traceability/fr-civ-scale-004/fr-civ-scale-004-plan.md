# Plan: FR-CIV-SCALE-004 -- Scale and performance

> Date: 2026-09-17
> FR: FR-CIV-SCALE-004
> Epic: FR-CIV-SCALE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Scale and performance logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/scale_budget.rs:18`
- `crates/voxel/src/scale_budget.rs:667`
- `crates/voxel/src/scale_budget.rs:798`
- `crates/voxel/src/scale_budget.rs:799`
- `docs/design/streaming-window.md:150`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:1113`
- `crates/voxel/src/scale_budget.rs:1115`
- `crates/voxel/src/scale_budget.rs:1127`
- `crates/voxel/src/scale_budget.rs:1156`
- `crates/voxel/src/scale_budget.rs:1174`
- `crates/voxel/src/scale_budget.rs:1201`

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
