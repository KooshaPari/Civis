# Plan: FR-SAVE-002 -- Save system

> Date: 2026-09-17
> FR: FR-SAVE-002
> Epic: FR-SAVE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/save-db/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Save system logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/replay.rs:297`
- `docs/specs/CIV-1000-save-load-persistence-spec.md:2801`

### Test Coverage
- `crates/engine/src/save.rs:278`
- `crates/engine/tests/fr_matrix_batch1.rs:17`
- `crates/engine/tests/fr_matrix_batch1.rs:322`
- `crates/engine/tests/fr_matrix_batch1.rs:323`
- `crates/engine/tests/fr_matrix_batch1.rs:324`
- `crates/engine/tests/fr_matrix_batch3.rs:151`
- `crates/engine/tests/fr_matrix_batch3.rs:152`

## Dependencies

- Epic: FR-SAVE
- Implementing crate: `crates/save-db/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p save-db`
2. `cargo test -p save-db`
3. `cargo clippy -p save-db`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
