# Intent: FR-CIV-ACT-001 -- Actor lifecycle

> Date: 2026-09-17
> FR: FR-CIV-ACT-001
> Epic: FR-CIV-ACT

## User Intent

The product owner requires Actor lifecycle as part of the FR-CIV-ACT epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Actor lifecycle is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-ACT-001 contributes to the overall simulation capability by addressing: Actor lifecycle.

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
2. The simulation runs without errors related to Actor lifecycle
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-act-001-spec.md` |
| ADR | `fr-civ-act-001-adr.md` |
| Research | `fr-civ-act-001-research.md` |
| Plan | `fr-civ-act-001-plan.md` |
| Implementing crate | `crates/engine/src/` |
