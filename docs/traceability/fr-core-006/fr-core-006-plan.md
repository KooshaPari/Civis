# Plan: FR-CORE-006 -- Core system

> Date: 2026-09-17
> FR: FR-CORE-006
> Epic: FR-CORE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Core system logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/hash_chain.rs:1`
- `docs/reference/agileplus-artifacts-index.md:41`
- `docs/reference/agileplus-artifacts-index.md:258`

### Test Coverage
- `crates/engine/src/hash_chain.rs:175`
- `crates/engine/tests/fr_matrix_batch1.rs:14`
- `crates/engine/tests/fr_matrix_batch1.rs:153`
- `crates/engine/tests/fr_matrix_batch1.rs:154`
- `crates/engine/tests/fr_matrix_batch1.rs:155`
- `crates/engine/tests/fr_matrix_batch3.rs:67`
- `crates/engine/tests/fr_matrix_batch3.rs:68`

## Dependencies

- Epic: FR-CORE
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
