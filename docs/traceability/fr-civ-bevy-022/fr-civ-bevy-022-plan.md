# Plan: FR-CIV-BEVY-022 -- Bevy rendering client

> Date: 2026-09-17
> FR: FR-CIV-BEVY-022
> Epic: FR-CIV-BEVY
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Bevy rendering client logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/bevy-ref/README.md:26`
- `docs/development-guide/p-w1-kickoff.md:133`
- `justfile:149`

### Test Coverage
- `clients/bevy-ref/src/live_focus.rs:123`
- `clients/bevy-ref/src/live_minimap.rs:182`
- `clients/bevy-ref/src/live_minimap.rs:207`
- `crates/build/tests/fr_matrix_batch12.rs:263`
- `crates/build/tests/fr_matrix_batch12.rs:266`

## Dependencies

- Epic: FR-CIV-BEVY
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
