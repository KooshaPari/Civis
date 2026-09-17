# Plan: FR-CIV-LEGENDS-003 -- Legend and narrative system

> Date: 2026-09-17
> FR: FR-CIV-LEGENDS-003
> Epic: FR-CIV-LEGENDS
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/legends/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Legend and narrative system logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/legends/src/rumor.rs:7`
- `crates/legends/src/rumor.rs:36`
- `crates/legends/src/rumor.rs:46`
- `crates/legends/src/rumor.rs:110`
- `crates/legends/src/rumor.rs:309`
- `crates/legends/src/rumor.rs:324`

### Test Coverage
- `crates/legends/src/rumor.rs:735`

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
