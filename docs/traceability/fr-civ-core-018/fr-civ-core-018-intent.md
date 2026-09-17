# Intent: FR-CIV-CORE-018 -- Core simulation engine

> Date: 2026-09-17
> FR: FR-CIV-CORE-018
> Epic: FR-CIV-CORE

## User Intent

The product owner requires Core simulation engine as part of the FR-CIV-CORE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Core simulation engine is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-CORE-018 contributes to the overall simulation capability by addressing: Core simulation engine.

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
2. The simulation runs without errors related to Core simulation engine
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-core-018-spec.md` |
| ADR | `fr-civ-core-018-adr.md` |
| Research | `fr-civ-core-018-research.md` |
| Plan | `fr-civ-core-018-plan.md` |
| Implementing crate | `crates/engine/src/` |
