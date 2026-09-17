# Plan: FR-CIV-CA-005 -- Combat and military

> Date: 2026-09-17
> FR: FR-CIV-CA-005
> Epic: FR-CIV-CA
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/ai/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Combat and military logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
> _To be implemented._

### Test Coverage
- `crates/voxel/src/fluid_ca.rs:1647`

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
