# Intent: FR-CIV-INSPECT-910 -- Inspection tools

> Date: 2026-09-17
> FR: FR-CIV-INSPECT-910
> Epic: FR-CIV-INSPECT

## User Intent

The product owner requires Inspection tools as part of the FR-CIV-INSPECT epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Inspection tools is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-INSPECT-910 contributes to the overall simulation capability by addressing: Inspection tools.

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
2. The simulation runs without errors related to Inspection tools
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-inspect-910-spec.md` |
| ADR | `fr-civ-inspect-910-adr.md` |
| Research | `fr-civ-inspect-910-research.md` |
| Plan | `fr-civ-inspect-910-plan.md` |
| Implementing crate | `crates/hud/src/` |
