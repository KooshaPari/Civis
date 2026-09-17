# Intent: FR-CIV-TERRAIN-006 -- Civ Terrain

> Date: 2026-09-17
> FR: FR-CIV-TERRAIN-006
> Epic: FR-CIV-TERRAIN

## User Intent

The product owner requires Civ Terrain as part of the FR-CIV-TERRAIN epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Civ Terrain is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-TERRAIN-006 contributes to the overall simulation capability by addressing: Civ Terrain.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/planet/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p planet` passes
2. The simulation runs without errors related to Civ Terrain
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-terrain-006-spec.md` |
| ADR | `fr-civ-terrain-006-adr.md` |
| Research | `fr-civ-terrain-006-research.md` |
| Plan | `fr-civ-terrain-006-plan.md` |
| Implementing crate | `crates/planet/src/` |
