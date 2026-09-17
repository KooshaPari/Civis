# Intent: FR-CIV-VEHICLE-046 -- Vehicle systems

> Date: 2026-09-17
> FR: FR-CIV-VEHICLE-046
> Epic: FR-CIV-VEHICLE

## User Intent

The product owner requires Vehicle systems as part of the FR-CIV-VEHICLE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Vehicle systems is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-VEHICLE-046 contributes to the overall simulation capability by addressing: Vehicle systems.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/civ-traffic/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p civ-traffic` passes
2. The simulation runs without errors related to Vehicle systems
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-vehicle-046-spec.md` |
| ADR | `fr-civ-vehicle-046-adr.md` |
| Research | `fr-civ-vehicle-046-research.md` |
| Plan | `fr-civ-vehicle-046-plan.md` |
| Implementing crate | `crates/civ-traffic/src/` |
