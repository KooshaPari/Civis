# Plan: FR-CIV-BUILD-001 -- Building tiers and construction

> Date: 2026-09-17
> FR: FR-CIV-BUILD-001
> Epic: FR-CIV-BUILD
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/physics-substrate/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Building tiers and construction logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/development-guide/fr-3d-additions.md:30`
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:272`

### Test Coverage
- `crates/build/src/lib.rs:515`
- `crates/build/src/lib.rs:726`
- `crates/build/tests/fr_matrix_batch12.rs:475`
- `crates/build/tests/fr_matrix_batch12.rs:478`

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
