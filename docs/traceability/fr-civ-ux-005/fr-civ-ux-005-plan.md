# Plan: FR-CIV-UX-005 -- User experience

> Date: 2026-09-17
> FR: FR-CIV-UX-005
> Epic: FR-CIV-UX
> Status: DRAFT

## Implementation Steps

1. **Research and Design** -- Review existing code in `crates/hud/src/` and finalize the ADR
2. **Core Implementation** -- Implement the User experience logic
3. **Integration** -- Wire into the Bevy ECS tick system and existing systems
4. **Testing** -- Add unit tests and integration tests
5. **Documentation** -- Update spec, ADR, and this plan with final decisions

### Referenced Source Files
- `clients/godot-ref/scripts/camera.gd:43`
- `clients/godot-ref/scripts/ui.tscn:107`
- `clients/godot-ref/scripts/ui.tscn:111`
- `clients/godot-ref/scripts/ui.tscn:115`
- `docs/development-guide/fr-unreal-agent-playbook.md:116`
- `web/dashboard/src/bottom_bar.tsx:434`
- `web/dashboard/src/store.tsx:295`

### Test Coverage
> _No test coverage yet._

## Dependencies

- Epic: FR-CIV-UX
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
