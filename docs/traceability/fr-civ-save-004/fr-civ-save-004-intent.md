# Intent: FR-CIV-SAVE-004 -- Save/load persistence

> Date: 2026-09-17
> FR: FR-CIV-SAVE-004
> Epic: FR-CIV-SAVE

## User Intent

The product owner requires Save/load persistence as part of the FR-CIV-SAVE epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Save/load persistence is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-SAVE-004 contributes to the overall simulation capability by addressing: Save/load persistence.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/save-db/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p save-db` passes
2. The simulation runs without errors related to Save/load persistence
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-save-004-spec.md` |
| ADR | `fr-civ-save-004-adr.md` |
| Research | `fr-civ-save-004-research.md` |
| Plan | `fr-civ-save-004-plan.md` |
| Implementing crate | `crates/save-db/src/` |
