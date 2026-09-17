# Plan: FR-CIV-INFOVIEW-901 -- Info views and inspectors

> Date: 2026-09-17
> FR: FR-CIV-INFOVIEW-901
> Epic: FR-CIV-INFOVIEW
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Info views and inspectors logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/bevy-ref/src/info_views.rs:18`
- `docs/agileplus/epics/civ-w4-perception.md:10`
- `docs/agileplus/epics/civ-w4-perception.md:28`
- `docs/agileplus/README.md:23`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:13`

### Test Coverage
- `clients/bevy-ref/src/info_views.rs:782`

## Dependencies

- Epic: FR-CIV-INFOVIEW
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
