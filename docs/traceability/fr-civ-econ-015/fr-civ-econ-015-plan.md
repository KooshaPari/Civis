# Plan: FR-CIV-ECON-015 -- Economy and joule allocation

> Date: 2026-09-17
> FR: FR-CIV-ECON-015
> Epic: FR-CIV-ECON
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/economy/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Economy and joule allocation logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/economy/src/chains.rs:2`

### Test Coverage
- `crates/economy/src/chains.rs:391`
- `crates/economy/src/chains.rs:414`
- `crates/economy/src/chains.rs:444`
- `crates/economy/src/chains.rs:466`
- `crates/economy/src/chains.rs:494`
- `crates/economy/src/chains.rs:537`
- `crates/economy/src/chains.rs:577`
- `crates/economy/src/chains.rs:598`

## Dependencies

- Epic: FR-CIV-ECON
- Implementing crate: `crates/economy/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p economy`
2. `cargo test -p economy`
3. `cargo clippy -p economy`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
