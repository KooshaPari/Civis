# Intent: FR-UX-020 -- User experience

> Date: 2026-09-17
> FR: FR-UX-020
> Epic: FR-UX

## User Intent

The product owner requires User experience as part of the FR-UX epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that User experience is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-UX-020 contributes to the overall simulation capability by addressing: User experience.

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
2. The simulation runs without errors related to User experience
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-ux-020-spec.md` |
| ADR | `fr-ux-020-adr.md` |
| Research | `fr-ux-020-research.md` |
| Plan | `fr-ux-020-plan.md` |
| Implementing crate | `crates/hud/src/` |
