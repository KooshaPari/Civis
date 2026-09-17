# Plan: FR-CIV-LIFE-020 -- Life simulation and needs

> Date: 2026-09-17
> FR: FR-CIV-LIFE-020
> Epic: FR-CIV-LIFE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/species/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Life simulation and needs logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `crates/engine/src/engine.rs:445`
- `crates/engine/src/engine.rs:2163`

### Test Coverage
- `crates/economy/src/stocks.rs:285`
- `crates/economy/src/stocks.rs:452`
- `crates/economy/tests/fr_matrix_batch7.rs:8`
- `crates/economy/tests/fr_matrix_batch7.rs:46`
- `crates/economy/tests/fr_matrix_batch7.rs:49`
- `crates/economy/tests/fr_matrix_batch7.rs:61`
- `crates/economy/tests/fr_matrix_batch7.rs:77`
- `crates/economy/tests/fr_matrix_batch7.rs:106`

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
