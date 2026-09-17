# Intent: FR-CIV-LIFE-001 -- Life simulation and needs

> Date: 2026-09-17
> FR: FR-CIV-LIFE-001
> Epic: FR-CIV-LIFE

## User Intent

The product owner requires Life simulation and needs as part of the FR-CIV-LIFE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Life simulation and needs is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-LIFE-001 contributes to the overall simulation capability by addressing: Life simulation and needs.

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
2. The simulation runs without errors related to Life simulation and needs
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-life-001-spec.md` |
| ADR | `fr-civ-life-001-adr.md` |
| Research | `fr-civ-life-001-research.md` |
| Plan | `fr-civ-life-001-plan.md` |
| Implementing crate | `crates/species/src/` |
