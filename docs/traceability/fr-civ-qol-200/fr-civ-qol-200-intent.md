# Intent: FR-CIV-QOL-200 -- Civ Qol

> Date: 2026-09-17
> FR: FR-CIV-QOL-200
> Epic: FR-CIV-QOL

## User Intent

The product owner requires Civ Qol as part of the FR-CIV-QOL epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Civ Qol is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-QOL-200 contributes to the overall simulation capability by addressing: Civ Qol.

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
2. The simulation runs without errors related to Civ Qol
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-qol-200-spec.md` |
| ADR | `fr-civ-qol-200-adr.md` |
| Research | `fr-civ-qol-200-research.md` |
| Plan | `fr-civ-qol-200-plan.md` |
| Implementing crate | `crates/hud/src/` |
