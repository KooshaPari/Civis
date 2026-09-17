# Intent: FR-CIV-BRUSH-05 -- Terrain brush tools

> Date: 2026-09-17
> FR: FR-CIV-BRUSH-05
> Epic: FR-CIV-BRUSH

## User Intent

The product owner requires Terrain brush tools as part of the FR-CIV-BRUSH epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Terrain brush tools is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-BRUSH-05 contributes to the overall simulation capability by addressing: Terrain brush tools.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/voxel/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p voxel` passes
2. The simulation runs without errors related to Terrain brush tools
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-brush-05-spec.md` |
| ADR | `fr-civ-brush-05-adr.md` |
| Research | `fr-civ-brush-05-research.md` |
| Plan | `fr-civ-brush-05-plan.md` |
| Implementing crate | `crates/voxel/src/` |
