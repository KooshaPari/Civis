# Plan: FR-REPLAY-001 -- Replay

> Date: 2026-09-17
> FR: FR-REPLAY-001
> Epic: FR-REPLAY
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Replay logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/engine.rs:1268`
- `crates/engine/src/replay_format.rs:1`
- `docs/reference/agileplus-artifacts-index.md:239`
- `docs/reference/agileplus-artifacts-index.md:311`

### Test Coverage
- `crates/engine/src/engine.rs:3206`
- `crates/engine/tests/fr_matrix_batch1.rs:15`
- `crates/engine/tests/fr_matrix_batch1.rs:191`
- `crates/engine/tests/fr_matrix_batch1.rs:192`
- `crates/engine/tests/fr_matrix_batch1.rs:193`
- `crates/engine/tests/fr_matrix_batch3.rs:87`
- `crates/engine/tests/fr_matrix_batch3.rs:88`

## Dependencies

- Epic: FR-REPLAY
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
