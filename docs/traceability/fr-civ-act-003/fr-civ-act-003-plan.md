# Plan: FR-CIV-ACT-003 -- Actor lifecycle

> Date: 2026-09-17
> FR: FR-CIV-ACT-003
> Epic: FR-CIV-ACT
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/engine/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Actor lifecycle logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/reference/REFERENCE_GAME_ANALYSIS.md:511`

### Test Coverage
> _No test coverage yet._

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
