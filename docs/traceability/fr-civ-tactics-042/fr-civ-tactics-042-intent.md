# Intent: FR-CIV-TACTICS-042 -- Tactics and strategy

> Date: 2026-09-17
> FR: FR-CIV-TACTICS-042
> Epic: FR-CIV-TACTICS

## User Intent

The product owner requires Tactics and strategy as part of the FR-CIV-TACTICS epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Tactics and strategy is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-TACTICS-042 contributes to the overall simulation capability by addressing: Tactics and strategy.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/tactics/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p tactics` passes
2. The simulation runs without errors related to Tactics and strategy
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-tactics-042-spec.md` |
| ADR | `fr-civ-tactics-042-adr.md` |
| Research | `fr-civ-tactics-042-research.md` |
| Plan | `fr-civ-tactics-042-plan.md` |
| Implementing crate | `crates/tactics/src/` |
