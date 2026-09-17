# Plan: FR-CIV-TACTICS-023 -- Tactics and strategy

> Date: 2026-09-17
> FR: FR-CIV-TACTICS-023
> Epic: FR-CIV-TACTICS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/tactics/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Tactics and strategy logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/tactics/src/doctrine_fitness.rs:1`
- `docs/design/warfare.md:122`
- `docs/development-guide/p-w1-kickoff.md:26`

### Test Coverage
- `crates/tactics/src/lib.rs:450`
- `crates/tactics/tests/fr_matrix_batch2.rs:179`
- `crates/tactics/tests/fr_matrix_batch2.rs:182`
- `crates/tactics/tests/fr_matrix_batch2.rs:183`
- `crates/tactics/tests/fr_matrix_batch5.rs:13`
- `crates/tactics/tests/fr_matrix_batch5.rs:188`
- `crates/tactics/tests/fr_matrix_batch5.rs:189`

## Dependencies

- Epic: FR-CIV-TACTICS
- Implementing crate: `crates/tactics/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p tactics`
2. `cargo test -p tactics`
3. `cargo clippy -p tactics`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
