# Plan: FR-CIV-DIPLO-002-SHADOW -- Diplomacy and treaties

> Date: 2026-09-17
> FR: FR-CIV-DIPLO-002-SHADOW
> Epic: FR-CIV-DIPLO
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/diplomacy/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Diplomacy and treaties logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `PLAN.md:209`
- `PLAN.md:210`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-CIV-DIPLO
- Implementing crate: `crates/diplomacy/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p diplomacy`
2. `cargo test -p diplomacy`
3. `cargo clippy -p diplomacy`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
