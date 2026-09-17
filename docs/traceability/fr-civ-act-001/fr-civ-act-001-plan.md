# Plan: FR-CIV-ACT-001 -- Actor lifecycle

> Date: 2026-09-17
> FR: FR-CIV-ACT-001
> Epic: FR-CIV-ACT
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Actor lifecycle logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/models/civ-sim/TECHNICAL_SPEC.md:2103`
- `docs/reference/FR_TRACKER.md:28`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:179`
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:509`
- `docs/reports/STATUS_REPORT.md:97`

### Test Coverage
- `crates/build/tests/fr_matrix_batch12.rs:115`
- `crates/build/tests/fr_matrix_batch12.rs:118`

## Dependencies

- Epic: FR-CIV-ACT
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
