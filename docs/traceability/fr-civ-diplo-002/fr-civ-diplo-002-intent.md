# Intent: FR-CIV-DIPLO-002 -- Diplomacy and treaties

> Date: 2026-09-17
> FR: FR-CIV-DIPLO-002
> Epic: FR-CIV-DIPLO

## User Intent

The product owner requires Diplomacy and treaties as part of the FR-CIV-DIPLO epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Diplomacy and treaties is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-DIPLO-002 contributes to the overall simulation capability by addressing: Diplomacy and treaties.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/diplomacy/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p diplomacy` passes
2. The simulation runs without errors related to Diplomacy and treaties
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-diplo-002-spec.md` |
| ADR | `fr-civ-diplo-002-adr.md` |
| Research | `fr-civ-diplo-002-research.md` |
| Plan | `fr-civ-diplo-002-plan.md` |
| Implementing crate | `crates/diplomacy/src/` |
