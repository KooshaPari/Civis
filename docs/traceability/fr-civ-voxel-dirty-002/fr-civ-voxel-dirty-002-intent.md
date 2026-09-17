# Intent: FR-CIV-VOXEL-DIRTY-002 -- Dirty voxel tracking

> Date: 2026-09-17
> FR: FR-CIV-VOXEL-DIRTY-002
> Epic: FR-CIV-VOXEL-DIRTY

## User Intent

The product owner requires Dirty voxel tracking as part of the FR-CIV-VOXEL-DIRTY epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Dirty voxel tracking is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-VOXEL-DIRTY-002 contributes to the overall simulation capability by addressing: Dirty voxel tracking.

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
2. The simulation runs without errors related to Dirty voxel tracking
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-voxel-dirty-002-spec.md` |
| ADR | `fr-civ-voxel-dirty-002-adr.md` |
| Research | `fr-civ-voxel-dirty-002-research.md` |
| Plan | `fr-civ-voxel-dirty-002-plan.md` |
| Implementing crate | `crates/voxel/src/` |
