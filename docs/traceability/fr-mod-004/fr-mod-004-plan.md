# Plan: FR-MOD-004 -- Modding

> Date: 2026-09-17
> FR: FR-MOD-004
> Epic: FR-MOD
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/mod-host/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Modding logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/engine.rs:1227`
- `crates/engine/src/replay.rs:51`
- `crates/engine/src/replay.rs:63`
- `crates/engine/src/replay.rs:82`
- `crates/engine/src/replay.rs:273`
- `crates/engine/src/replay.rs:285`
- `crates/mod-host/src/lib.rs:204`
- `crates/mod-host/src/lib.rs:217`

### Test Coverage
- `crates/engine/src/scenario.rs:347`
- `crates/engine/tests/fr_matrix_batch1.rs:16`
- `crates/engine/tests/fr_matrix_batch1.rs:263`
- `crates/engine/tests/fr_matrix_batch1.rs:264`
- `crates/engine/tests/fr_matrix_batch1.rs:265`
- `crates/engine/tests/fr_matrix_batch1.rs:303`
- `crates/engine/tests/fr_matrix_batch3.rs:120`
- `crates/engine/tests/fr_matrix_batch3.rs:121`

## Dependencies

- Epic: FR-MOD
- Implementing crate: `crates/mod-host/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p mod-host`
2. `cargo test -p mod-host`
3. `cargo clippy -p mod-host`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
