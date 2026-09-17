# Intent: FR-CIV-RES-001 -- Civ Res

> Date: 2026-09-17
> FR: FR-CIV-RES-001
> Epic: FR-CIV-RES

## User Intent

The product owner requires Civ Res as part of the FR-CIV-RES epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Civ Res is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-RES-001 contributes to the overall simulation capability by addressing: Civ Res.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/economy/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p economy` passes
2. The simulation runs without errors related to Civ Res
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-res-001-spec.md` |
| ADR | `fr-civ-res-001-adr.md` |
| Research | `fr-civ-res-001-research.md` |
| Plan | `fr-civ-res-001-plan.md` |
| Implementing crate | `crates/economy/src/` |
