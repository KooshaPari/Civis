# Intent: FR-INST-004 -- Institution

> Date: 2026-09-17
> FR: FR-INST-004
> Epic: FR-INST

## User Intent

The product owner requires Institution as part of the FR-INST epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Institution is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-INST-004 contributes to the overall simulation capability by addressing: Institution.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/civ-institutions/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p civ-institutions` passes
2. The simulation runs without errors related to Institution
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-inst-004-spec.md` |
| ADR | `fr-inst-004-adr.md` |
| Research | `fr-inst-004-research.md` |
| Plan | `fr-inst-004-plan.md` |
| Implementing crate | `crates/civ-institutions/src/` |
