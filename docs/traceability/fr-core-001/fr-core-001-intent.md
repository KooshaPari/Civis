# Intent: FR-CORE-001 -- Core system

> Date: 2026-09-17
> FR: FR-CORE-001
> Epic: FR-CORE

## User Intent

The product owner requires Core system as part of the FR-CORE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Core system is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CORE-001 contributes to the overall simulation capability by addressing: Core system.

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
2. The simulation runs without errors related to Core system
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-core-001-spec.md` |
| ADR | `fr-core-001-adr.md` |
| Research | `fr-core-001-research.md` |
| Plan | `fr-core-001-plan.md` |
| Implementing crate | `crates/engine/src/` |
