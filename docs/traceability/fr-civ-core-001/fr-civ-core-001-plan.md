# Plan: FR-CIV-CORE-001 -- Core simulation engine

> Date: 2026-09-17
> FR: FR-CIV-CORE-001
> Epic: FR-CIV-CORE
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Core simulation engine logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/AGILE_WORKSTREAM.md:372`
- `docs/AGILE_WORKSTREAM.md:444`
- `docs/AGILE_WORKSTREAM.md:455`
- `docs/AGILE_WORKSTREAM.md:512`
- `docs/AGILE_WORKSTREAM.md:586`
- `docs/AGILE_WORKSTREAM.md:590`
- `docs/AGILE_WORKSTREAM.md:618`
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2104`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:692`
- `crates/build/tests/fr_matrix_batch12.rs:695`

## Dependencies

- Epic: FR-CIV-CORE
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
