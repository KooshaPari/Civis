# Plan: FR-CIV-HUD-004 -- HUD overlay

> Date: 2026-09-17
> FR: FR-CIV-HUD-004
> Epic: FR-CIV-HUD
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the HUD overlay logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/voxel/src/hud.rs:10`
- `crates/voxel/src/hud.rs:509`
- `docs/reference/agileplus-artifacts-index.md:201`
- `docs/reference/agileplus-artifacts-index.md:303`

### Test Coverage
- `crates/voxel/src/hud.rs:858`
- `crates/voxel/src/hud.rs:860`
- `crates/voxel/src/hud.rs:872`
- `crates/voxel/src/hud.rs:889`

## Dependencies

- Epic: FR-CIV-HUD
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
