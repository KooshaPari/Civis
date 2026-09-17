# Intent: FR-CIV-HUD-001 -- HUD overlay

> Date: 2026-09-17
> FR: FR-CIV-HUD-001
> Epic: FR-CIV-HUD

## User Intent

The product owner requires HUD overlay as part of the FR-CIV-HUD epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that HUD overlay is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-HUD-001 contributes to the overall simulation capability by addressing: HUD overlay.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/hud/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p hud` passes
2. The simulation runs without errors related to HUD overlay
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-hud-001-spec.md` |
| ADR | `fr-civ-hud-001-adr.md` |
| Research | `fr-civ-hud-001-research.md` |
| Plan | `fr-civ-hud-001-plan.md` |
| Implementing crate | `crates/hud/src/` |
