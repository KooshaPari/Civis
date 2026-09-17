# Intent: FR-CIV-WAR-021 -- War and conflict

> Date: 2026-09-17
> FR: FR-CIV-WAR-021
> Epic: FR-CIV-WAR

## User Intent

The product owner requires War and conflict as part of the FR-CIV-WAR epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that War and conflict is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-WAR-021 contributes to the overall simulation capability by addressing: War and conflict.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/tactics/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p tactics` passes
2. The simulation runs without errors related to War and conflict
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-war-021-spec.md` |
| ADR | `fr-civ-war-021-adr.md` |
| Research | `fr-civ-war-021-research.md` |
| Plan | `fr-civ-war-021-plan.md` |
| Implementing crate | `crates/tactics/src/` |
