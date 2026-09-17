# Intent: FR-CIV-BIO-001 -- Biological simulation

> Date: 2026-09-17
> FR: FR-CIV-BIO-001
> Epic: FR-CIV-BIO

## User Intent

The product owner requires Biological simulation as part of the FR-CIV-BIO epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Biological simulation is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-BIO-001 contributes to the overall simulation capability by addressing: Biological simulation.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/species/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p species` passes
2. The simulation runs without errors related to Biological simulation
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-bio-001-spec.md` |
| ADR | `fr-civ-bio-001-adr.md` |
| Research | `fr-civ-bio-001-research.md` |
| Plan | `fr-civ-bio-001-plan.md` |
| Implementing crate | `crates/species/src/` |
