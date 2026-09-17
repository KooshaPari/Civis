# Intent: FR-CIV-GEO-007 -- Geological systems

> Date: 2026-09-17
> FR: FR-CIV-GEO-007
> Epic: FR-CIV-GEO

## User Intent

The product owner requires Geological systems as part of the FR-CIV-GEO epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Geological systems is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-GEO-007 contributes to the overall simulation capability by addressing: Geological systems.

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
2. The simulation runs without errors related to Geological systems
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-geo-007-spec.md` |
| ADR | `fr-civ-geo-007-adr.md` |
| Research | `fr-civ-geo-007-research.md` |
| Plan | `fr-civ-geo-007-plan.md` |
| Implementing crate | `crates/planet/src/` |
