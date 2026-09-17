# Intent: FR-CIV-INFOVIEW-906 -- Info views and inspectors

> Date: 2026-09-17
> FR: FR-CIV-INFOVIEW-906
> Epic: FR-CIV-INFOVIEW

## User Intent

The product owner requires Info views and inspectors as part of the FR-CIV-INFOVIEW epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Info views and inspectors is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-INFOVIEW-906 contributes to the overall simulation capability by addressing: Info views and inspectors.

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
2. The simulation runs without errors related to Info views and inspectors
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-infoview-906-spec.md` |
| ADR | `fr-civ-infoview-906-adr.md` |
| Research | `fr-civ-infoview-906-research.md` |
| Plan | `fr-civ-infoview-906-plan.md` |
| Implementing crate | `crates/hud/src/` |
