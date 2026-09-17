# Intent: FR-CIV-ACTOR-001 -- Actor and citizen lifecycle

> Date: 2026-09-17
> FR: FR-CIV-ACTOR-001
> Epic: FR-CIV-ACTOR

## User Intent

The product owner requires Actor and citizen lifecycle as part of the FR-CIV-ACTOR epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Actor and citizen lifecycle is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-ACTOR-001 contributes to the overall simulation capability by addressing: Actor and citizen lifecycle.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/species/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p species` passes
2. The simulation runs without errors related to Actor and citizen lifecycle
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-actor-001-spec.md` |
| ADR | `fr-civ-actor-001-adr.md` |
| Research | `fr-civ-actor-001-research.md` |
| Plan | `fr-civ-actor-001-plan.md` |
| Implementing crate | `crates/species/src/` |
