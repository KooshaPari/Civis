# Plan: FR-ECON-001 -- Economics

> Date: 2026-09-17
> FR: FR-ECON-001
> Epic: FR-ECON
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/economy/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Economics logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/economy/src/lib.rs:106`
- `crates/engine/src/engine.rs:2011`
- `docs/reference/agileplus-artifacts-index.md:57`
- `docs/reference/agileplus-artifacts-index.md:89`
- `docs/reference/agileplus-artifacts-index.md:260`

### Test Coverage
- `crates/economy/src/lib.rs:308`
- `crates/economy/tests/fr_matrix_batch7.rs:9`
- `crates/economy/tests/fr_matrix_batch7.rs:135`
- `crates/economy/tests/fr_matrix_batch7.rs:138`
- `crates/economy/tests/fr_matrix_batch7.rs:154`
- `crates/economy/tests/fr_matrix_batch7.rs:165`

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
