# Plan: FR-CIV-SCALE-003 -- Scale and performance

> Date: 2026-09-17
> FR: FR-CIV-SCALE-003
> Epic: FR-CIV-SCALE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Scale and performance logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/scale_budget.rs:14`
- `crates/voxel/src/scale_budget.rs:404`
- `crates/voxel/src/scale_budget.rs:494`
- `crates/voxel/src/scale_budget.rs:495`
- `docs/design/streaming-window.md:225`

### Test Coverage
- `crates/voxel/src/scale_budget.rs:1033`
- `crates/voxel/src/scale_budget.rs:1035`
- `crates/voxel/src/scale_budget.rs:1058`
- `crates/voxel/src/scale_budget.rs:1077`
- `crates/voxel/src/scale_budget.rs:1095`

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
