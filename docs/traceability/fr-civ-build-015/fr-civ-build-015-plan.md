# Plan: FR-CIV-BUILD-015 -- Building tiers and construction

> Date: 2026-09-17
> FR: FR-CIV-BUILD-015
> Epic: FR-CIV-BUILD
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/physics-substrate/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Building tiers and construction logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
> _To be implemented._

### Test Coverage
- `crates/build/src/lib.rs:842`

## Dependencies

- Epic: FR-CIV-BUILD
- Implementing crate: `crates/physics-substrate/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p physics-substrate`
2. `cargo test -p physics-substrate`
3. `cargo clippy -p physics-substrate`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
