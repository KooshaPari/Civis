# Intent: FR-ECON-002 -- Economics

> Date: 2026-09-17
> FR: FR-ECON-002
> Epic: FR-ECON

## User Intent

The product owner requires Economics as part of the FR-ECON epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Economics is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-ECON-002 contributes to the overall simulation capability by addressing: Economics.

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
2. The simulation runs without errors related to Economics
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-econ-002-spec.md` |
| ADR | `fr-econ-002-adr.md` |
| Research | `fr-econ-002-research.md` |
| Plan | `fr-econ-002-plan.md` |
| Implementing crate | `crates/economy/src/` |
