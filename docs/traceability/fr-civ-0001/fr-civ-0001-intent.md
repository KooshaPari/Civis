# Intent: FR-CIV-0001 -- Core civilisation simulation

> Date: 2026-09-17
> FR: FR-CIV-0001
> Epic: FR-CIV

## User Intent

The product owner requires Core civilisation simulation as part of the FR-CIV epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Core civilisation simulation is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-0001 contributes to the overall simulation capability by addressing: Core civilisation simulation.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/engine/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p engine` passes
2. The simulation runs without errors related to Core civilisation simulation
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-0001-spec.md` |
| ADR | `fr-civ-0001-adr.md` |
| Research | `fr-civ-0001-research.md` |
| Plan | `fr-civ-0001-plan.md` |
| Implementing crate | `crates/engine/src/` |
