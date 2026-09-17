# Plan: FR-CIV-0001 -- Core civilisation simulation

> Date: 2026-09-17
> FR: FR-CIV-0001
> Epic: FR-CIV
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Core civilisation simulation logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/guides/GIT_WORKTREE_GUIDE.md:151`
- `PLAN.md:16`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:102`
- `crates/build/tests/fr_matrix_batch12.rs:105`

## Dependencies

- Epic: FR-CIV
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
