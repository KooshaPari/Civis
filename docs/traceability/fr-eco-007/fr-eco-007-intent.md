# Intent: FR-ECO-007 -- Economy

> Date: 2026-09-17
> FR: FR-ECO-007
> Epic: FR-ECO

## User Intent

The product owner requires Economy as part of the FR-ECO epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Economy is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-ECO-007 contributes to the overall simulation capability by addressing: Economy.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/economy/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p economy` passes
2. The simulation runs without errors related to Economy
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-eco-007-spec.md` |
| ADR | `fr-eco-007-adr.md` |
| Research | `fr-eco-007-research.md` |
| Plan | `fr-eco-007-plan.md` |
| Implementing crate | `crates/economy/src/` |
