# Intent: FR-CIV-SPECIES-006 -- Species definitions

> Date: 2026-09-17
> FR: FR-CIV-SPECIES-006
> Epic: FR-CIV-SPECIES

## User Intent

The product owner requires Species definitions as part of the FR-CIV-SPECIES epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Species definitions is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-SPECIES-006 contributes to the overall simulation capability by addressing: Species definitions.

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
2. The simulation runs without errors related to Species definitions
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-species-006-spec.md` |
| ADR | `fr-civ-species-006-adr.md` |
| Research | `fr-civ-species-006-research.md` |
| Plan | `fr-civ-species-006-plan.md` |
| Implementing crate | `crates/species/src/` |
