# Plan: FR-CIV-INSPECT-920 -- Inspection tools

> Date: 2026-09-17
> FR: FR-CIV-INSPECT-920
> Epic: FR-CIV-INSPECT
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Inspection tools logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `docs/agileplus/epics/civ-w4-perception.md:22`
- `docs/agileplus/epics/civ-w4-perception.md:37`
- `docs/agileplus/README.md:23`
- `docs/design/onboarding-qol.md:138`
- `docs/specs/requirements/FR-CIV-INSPECT.md:16`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-CIV-INSPECT
- Implementing crate: `crates/hud/src/`
- Engine core: `crates/engine/src/`

## Verification

1. `cargo check -p hud`
2. `cargo test -p hud`
3. `cargo clippy -p hud`
4. Manual verification in the simulation runtime

## Estimated Effort

- Implementation: TBD
- Testing: TBD
- Total: TBD
