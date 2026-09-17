# Intent: FR-SOC-CIV-002 -- Social civilisation

> Date: 2026-09-17
> FR: FR-SOC-CIV-002
> Epic: FR-SOC-CIV

## User Intent

The product owner requires Social civilisation as part of the FR-SOC-CIV epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Social civilisation is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-SOC-CIV-002 contributes to the overall simulation capability by addressing: Social civilisation.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/civ-institutions/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p civ-institutions` passes
2. The simulation runs without errors related to Social civilisation
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-soc-civ-002-spec.md` |
| ADR | `fr-soc-civ-002-adr.md` |
| Research | `fr-soc-civ-002-research.md` |
| Plan | `fr-soc-civ-002-plan.md` |
| Implementing crate | `crates/civ-institutions/src/` |
