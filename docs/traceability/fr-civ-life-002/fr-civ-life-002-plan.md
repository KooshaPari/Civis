# Plan: FR-CIV-LIFE-002 -- Life simulation and needs

> Date: 2026-09-17
> FR: FR-CIV-LIFE-002
> Epic: FR-CIV-LIFE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/species/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Life simulation and needs logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/needs/src/lib.rs:14`

### Test Coverage
- `crates/needs/src/lib.rs:485`
- `crates/needs/src/lib.rs:528`
- `crates/needs/src/lib.rs:607`
- `crates/needs/src/lib.rs:648`
- `crates/needs/src/lib.rs:777`
- `crates/needs/src/lib.rs:819`
- `crates/needs/src/lib.rs:857`

## Dependencies

- Epic: FR-CIV-LIFE
- Implementing crate: `crates/species/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p species`
2. `cargo test -p species`
3. `cargo clippy -p species`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
