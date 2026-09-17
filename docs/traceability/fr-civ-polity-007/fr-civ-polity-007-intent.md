# Intent: FR-CIV-POLITY-007 -- Polity system

> Date: 2026-09-17
> FR: FR-CIV-POLITY-007
> Epic: FR-CIV-POLITY

## User Intent

The product owner requires Polity system as part of the FR-CIV-POLITY epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Polity system is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-POLITY-007 contributes to the overall simulation capability by addressing: Polity system.

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
2. The simulation runs without errors related to Polity system
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-polity-007-spec.md` |
| ADR | `fr-civ-polity-007-adr.md` |
| Research | `fr-civ-polity-007-research.md` |
| Plan | `fr-civ-polity-007-plan.md` |
| Implementing crate | `crates/diplomacy/src/` |
