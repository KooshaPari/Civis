# Intent: FR-CIV-ECON-004 -- Economy and joule allocation

> Date: 2026-09-17
> FR: FR-CIV-ECON-004
> Epic: FR-CIV-ECON

## User Intent

The product owner requires Economy and joule allocation as part of the FR-CIV-ECON epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Economy and joule allocation is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-ECON-004 contributes to the overall simulation capability by addressing: Economy and joule allocation.

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
2. The simulation runs without errors related to Economy and joule allocation
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-econ-004-spec.md` |
| ADR | `fr-civ-econ-004-adr.md` |
| Research | `fr-civ-econ-004-research.md` |
| Plan | `fr-civ-econ-004-plan.md` |
| Implementing crate | `crates/economy/src/` |
