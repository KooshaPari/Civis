# Plan: FR-CIV-VOXEL-020 -- Voxel rendering

> Date: 2026-09-17
> FR: FR-CIV-VOXEL-020
> Epic: FR-CIV-VOXEL
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/voxel/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Voxel rendering logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/bevy-ref/src/bin/standalone.rs:194`
- `clients/bevy-ref/src/voxel_stream.rs:13`
- `crates/voxel/src/stream.rs:32`
- `docs/guides/voxel-emergent-vision-and-migration.md:94`
- `docs/guides/voxel-emergent-vision-and-migration.md:119`
- `docs/guides/voxel-emergent-vision-and-migration.md:123`

### Test Coverage
- `clients/bevy-ref/src/voxel_stream.rs:352`
- `clients/bevy-ref/src/voxel_stream.rs:364`

## Dependencies

- Epic: FR-CIV-VOXEL
- Implementing crate: `crates/voxel/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p voxel`
2. `cargo test -p voxel`
3. `cargo clippy -p voxel`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
