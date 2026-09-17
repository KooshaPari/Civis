# Plan: FR-CIV-INSPECT-910 -- Inspection tools

> Date: 2026-09-17
> FR: FR-CIV-INSPECT-910
> Epic: FR-CIV-INSPECT
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Inspection tools logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/bevy-ref/src/inspect.rs:13`
- `clients/bevy-ref/src/inspect.rs:104`
- `clients/bevy-ref/src/inspect.rs:323`
- `docs/agileplus/epics/civ-w4-perception.md:21`
- `docs/agileplus/epics/civ-w4-perception.md:37`
- `docs/agileplus/README.md:23`
- `docs/specs/requirements/FR-CIV-INSPECT.md:15`

### Test Coverage
- `clients/bevy-ref/src/inspect.rs:392`

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
