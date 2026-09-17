# Plan: FR-CIV-TACTICS-022 -- Tactics and strategy

> Date: 2026-09-17
> FR: FR-CIV-TACTICS-022
> Epic: FR-CIV-TACTICS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/tactics/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Tactics and strategy logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/tactics/src/war_bridge.rs:1`
- `crates/tactics/src/war_bridge.rs:186`
- `docs/development-guide/fr-3d-additions.md:91`
- `docs/development-guide/p-w1-kickoff.md:25`

### Test Coverage
- `crates/tactics/src/lib.rs:320`
- `crates/tactics/src/war_bridge.rs:289`
- `crates/tactics/tests/fr_matrix_batch2.rs:141`
- `crates/tactics/tests/fr_matrix_batch2.rs:144`
- `crates/tactics/tests/fr_matrix_batch2.rs:145`
- `crates/tactics/tests/fr_matrix_batch5.rs:12`
- `crates/tactics/tests/fr_matrix_batch5.rs:170`
- `crates/tactics/tests/fr_matrix_batch5.rs:171`

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
