# Intent: FR-CIV-FOG-001 -- Fog of war

> Date: 2026-09-17
> FR: FR-CIV-FOG-001
> Epic: FR-CIV-FOG

## User Intent

The product owner requires Fog of war as part of the FR-CIV-FOG epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Fog of war is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-FOG-001 contributes to the overall simulation capability by addressing: Fog of war.

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
2. The simulation runs without errors related to Fog of war
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-fog-001-spec.md` |
| ADR | `fr-civ-fog-001-adr.md` |
| Research | `fr-civ-fog-001-research.md` |
| Plan | `fr-civ-fog-001-plan.md` |
| Implementing crate | `crates/engine/src/` |
