# Plan: FR-CIV-TACTICS-031 -- Tactics and strategy

> Date: 2026-09-17
> FR: FR-CIV-TACTICS-031
> Epic: FR-CIV-TACTICS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/tactics/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Tactics and strategy logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/tactics/src/movement.rs:1`
- `docs/development-guide/p-w1-kickoff.md:30`

### Test Coverage
- `crates/tactics/src/lib.rs:419`
- `crates/tactics/tests/fr_matrix_batch2.rs:256`
- `crates/tactics/tests/fr_matrix_batch2.rs:259`
- `crates/tactics/tests/fr_matrix_batch2.rs:260`

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
