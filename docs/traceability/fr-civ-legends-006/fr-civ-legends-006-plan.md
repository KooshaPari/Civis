# Plan: FR-CIV-LEGENDS-006 -- Legend and narrative system

> Date: 2026-09-17
> FR: FR-CIV-LEGENDS-006
> Epic: FR-CIV-LEGENDS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/legends/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Legend and narrative system logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/legends/src/graph.rs:77`
- `crates/legends/src/graph.rs:89`
- `crates/legends/src/graph.rs:589`
- `crates/legends/src/graph.rs:607`

### Test Coverage
- `crates/legends/tests/fr_legends_completion.rs:8`
- `crates/legends/tests/fr_legends_completion.rs:376`
- `crates/legends/tests/fr_legends_completion.rs:379`
- `crates/legends/tests/fr_legends_completion.rs:417`
- `crates/legends/tests/fr_legends_completion.rs:431`
- `crates/legends/tests/fr_legends_completion.rs:468`
- `crates/legends/tests/fr_legends_completion.rs:476`

## Dependencies

- Epic: FR-CIV-LEGENDS
- Implementing crate: `crates/legends/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p legends`
2. `cargo test -p legends`
3. `cargo clippy -p legends`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
