# Intent: FR-SOC-HLT-005 -- Social health

> Date: 2026-09-17
> FR: FR-SOC-HLT-005
> Epic: FR-SOC-HLT

## User Intent

The product owner requires Social health as part of the FR-SOC-HLT epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Social health is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-SOC-HLT-005 contributes to the overall simulation capability by addressing: Social health.

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
2. The simulation runs without errors related to Social health
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-soc-hlt-005-spec.md` |
| ADR | `fr-soc-hlt-005-adr.md` |
| Research | `fr-soc-hlt-005-research.md` |
| Plan | `fr-soc-hlt-005-plan.md` |
| Implementing crate | `crates/civ-institutions/src/` |
