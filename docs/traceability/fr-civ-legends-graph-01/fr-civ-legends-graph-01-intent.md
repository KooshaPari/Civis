# Intent: FR-CIV-LEGENDS-GRAPH-01 -- Civ Legends Graph

> Date: 2026-09-17
> FR: FR-CIV-LEGENDS-GRAPH-01
> Epic: FR-CIV-LEGENDS-GRAPH

## User Intent

The product owner requires Civ Legends Graph as part of the FR-CIV-LEGENDS-GRAPH epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Civ Legends Graph is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-LEGENDS-GRAPH-01 contributes to the overall simulation capability by addressing: Civ Legends Graph.

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
2. The simulation runs without errors related to Civ Legends Graph
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-legends-graph-01-spec.md` |
| ADR | `fr-civ-legends-graph-01-adr.md` |
| Research | `fr-civ-legends-graph-01-research.md` |
| Plan | `fr-civ-legends-graph-01-plan.md` |
| Implementing crate | `crates/legends/src/` |
