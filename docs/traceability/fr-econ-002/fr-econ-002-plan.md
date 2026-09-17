# Plan: FR-ECON-002 -- Economics

> Date: 2026-09-17
> FR: FR-ECON-002
> Epic: FR-ECON
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/economy/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Economics logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:261`

### Test Coverage
- `crates/economy/src/allocator.rs:765`
- `crates/economy/src/allocator.rs:816`
- `crates/economy/src/lib.rs:381`
- `crates/economy/tests/fr_matrix_batch7.rs:10`
- `crates/economy/tests/fr_matrix_batch7.rs:180`
- `crates/economy/tests/fr_matrix_batch7.rs:183`
- `crates/economy/tests/fr_matrix_batch7.rs:223`

## Dependencies

- Epic: FR-ECON
- Implementing crate: `crates/economy/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p economy`
2. `cargo test -p economy`
3. `cargo clippy -p economy`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
