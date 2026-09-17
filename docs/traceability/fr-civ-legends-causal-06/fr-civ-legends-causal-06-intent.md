# Intent: FR-CIV-LEGENDS-CAUSAL-06 -- Civ Legends Causal

> Date: 2026-09-17
> FR: FR-CIV-LEGENDS-CAUSAL-06
> Epic: FR-CIV-LEGENDS-CAUSAL

## User Intent

The product owner requires Civ Legends Causal as part of the FR-CIV-LEGENDS-CAUSAL epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Civ Legends Causal is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-LEGENDS-CAUSAL-06 contributes to the overall simulation capability by addressing: Civ Legends Causal.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/legends/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p legends` passes
2. The simulation runs without errors related to Civ Legends Causal
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-legends-causal-06-spec.md` |
| ADR | `fr-civ-legends-causal-06-adr.md` |
| Research | `fr-civ-legends-causal-06-research.md` |
| Plan | `fr-civ-legends-causal-06-plan.md` |
| Implementing crate | `crates/legends/src/` |
