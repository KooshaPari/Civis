# Intent: FR-CIV-PBR-006 -- Physically based rendering

> Date: 2026-09-17
> FR: FR-CIV-PBR-006
> Epic: FR-CIV-PBR

## User Intent

The product owner requires Physically based rendering as part of the FR-CIV-PBR epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Physically based rendering is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-PBR-006 contributes to the overall simulation capability by addressing: Physically based rendering.

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
2. The simulation runs without errors related to Physically based rendering
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-pbr-006-spec.md` |
| ADR | `fr-civ-pbr-006-adr.md` |
| Research | `fr-civ-pbr-006-research.md` |
| Plan | `fr-civ-pbr-006-plan.md` |
| Implementing crate | `crates/engine/src/` |
