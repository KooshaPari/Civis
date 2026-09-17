# Plan: FR-CIV-TACTICS-020 -- Tactics and strategy

> Date: 2026-09-17
> FR: FR-CIV-TACTICS-020
> Epic: FR-CIV-TACTICS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/tactics/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Tactics and strategy logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/tactics/src/los.rs:1`
- `docs/development-guide/fr-3d-additions.md:89`
- `docs/development-guide/p-w1-kickoff.md:23`

### Test Coverage
- `crates/tactics/src/lib.rs:290`
- `crates/tactics/src/los.rs:137`
- `crates/tactics/src/los.rs:176`
- `crates/tactics/src/los.rs:235`
- `crates/tactics/tests/fr_matrix_batch2.rs:111`
- `crates/tactics/tests/fr_matrix_batch2.rs:114`
- `crates/tactics/tests/fr_matrix_batch2.rs:115`
- `crates/tactics/tests/fr_matrix_batch5.rs:10`

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
