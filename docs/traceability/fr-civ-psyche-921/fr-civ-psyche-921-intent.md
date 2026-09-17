# Intent: FR-CIV-PSYCHE-921 -- Psychological and social modelling

> Date: 2026-09-17
> FR: FR-CIV-PSYCHE-921
> Epic: FR-CIV-PSYCHE

## User Intent

The product owner requires Psychological and social modelling as part of the FR-CIV-PSYCHE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Psychological and social modelling is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-PSYCHE-921 contributes to the overall simulation capability by addressing: Psychological and social modelling.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/needs/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p needs` passes
2. The simulation runs without errors related to Psychological and social modelling
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-psyche-921-spec.md` |
| ADR | `fr-civ-psyche-921-adr.md` |
| Research | `fr-civ-psyche-921-research.md` |
| Plan | `fr-civ-psyche-921-plan.md` |
| Implementing crate | `crates/needs/src/` |
