# Intent: FR-CIV-CA-009 -- Combat and military

> Date: 2026-09-17
> FR: FR-CIV-CA-009
> Epic: FR-CIV-CA

## User Intent

The product owner requires Combat and military as part of the FR-CIV-CA epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Combat and military is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-CA-009 contributes to the overall simulation capability by addressing: Combat and military.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/ai/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p ai` passes
2. The simulation runs without errors related to Combat and military
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-ca-009-spec.md` |
| ADR | `fr-civ-ca-009-adr.md` |
| Research | `fr-civ-ca-009-research.md` |
| Plan | `fr-civ-ca-009-plan.md` |
| Implementing crate | `crates/ai/src/` |
