# Plan: FR-CIV-INFOVIEW-900 -- Info views and inspectors

> Date: 2026-09-17
> FR: FR-CIV-INFOVIEW-900
> Epic: FR-CIV-INFOVIEW
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the Info views and inspectors logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/bevy-ref/src/info_views.rs:17`
- `clients/bevy-ref/src/info_views.rs:380`
- `docs/agileplus/epics/civ-w4-perception.md:9`
- `docs/agileplus/epics/civ-w4-perception.md:28`
- `docs/agileplus/README.md:23`
- `docs/design/info-views.md:214`
- `docs/design/master-roadmap.md:22`
- `docs/specs/requirements/FR-CIV-INFOVIEW.md:12`

### Test Coverage
- `clients/bevy-ref/src/info_views.rs:738`
- `clients/bevy-ref/src/info_views.rs:759`
- `clients/bevy-ref/src/info_views.rs:773`

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
