# Intent: FR-CIV-SOCIAL-002 -- Social systems

> Date: 2026-09-17
> FR: FR-CIV-SOCIAL-002
> Epic: FR-CIV-SOCIAL

## User Intent

The product owner requires Social systems as part of the FR-CIV-SOCIAL epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Social systems is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-SOCIAL-002 contributes to the overall simulation capability by addressing: Social systems.

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
2. The simulation runs without errors related to Social systems
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-social-002-spec.md` |
| ADR | `fr-civ-social-002-adr.md` |
| Research | `fr-civ-social-002-research.md` |
| Plan | `fr-civ-social-002-plan.md` |
| Implementing crate | `crates/civ-institutions/src/` |
