# Intent: FR-CIV-VOXEL-020 -- Voxel rendering

> Date: 2026-09-17
> FR: FR-CIV-VOXEL-020
> Epic: FR-CIV-VOXEL

## User Intent

The product owner requires Voxel rendering as part of the FR-CIV-VOXEL epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Voxel rendering is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-VOXEL-020 contributes to the overall simulation capability by addressing: Voxel rendering.

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
2. The simulation runs without errors related to Voxel rendering
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-voxel-020-spec.md` |
| ADR | `fr-civ-voxel-020-adr.md` |
| Research | `fr-civ-voxel-020-research.md` |
| Plan | `fr-civ-voxel-020-plan.md` |
| Implementing crate | `crates/voxel/src/` |
