# Intent: FR-CIV-ROAD-920 -- Road and path systems

> Date: 2026-09-17
> FR: FR-CIV-ROAD-920
> Epic: FR-CIV-ROAD

## User Intent

The product owner requires Road and path systems as part of the FR-CIV-ROAD epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Road and path systems is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-ROAD-920 contributes to the overall simulation capability by addressing: Road and path systems.

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
2. The simulation runs without errors related to Road and path systems
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-road-920-spec.md` |
| ADR | `fr-civ-road-920-adr.md` |
| Research | `fr-civ-road-920-research.md` |
| Plan | `fr-civ-road-920-plan.md` |
| Implementing crate | `crates/civ-traffic/src/` |
