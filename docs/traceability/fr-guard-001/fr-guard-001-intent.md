# Intent: FR-GUARD-001 -- Guard rails

> Date: 2026-09-17
> FR: FR-GUARD-001
> Epic: FR-GUARD

## User Intent

The product owner requires Guard rails as part of the FR-GUARD epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Guard rails is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-GUARD-001 contributes to the overall simulation capability by addressing: Guard rails.

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
2. The simulation runs without errors related to Guard rails
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-guard-001-spec.md` |
| ADR | `fr-guard-001-adr.md` |
| Research | `fr-guard-001-research.md` |
| Plan | `fr-guard-001-plan.md` |
| Implementing crate | `crates/engine/src/` |
