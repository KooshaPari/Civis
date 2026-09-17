# Intent: FR-SOC-DET-002 -- Social determinism

> Date: 2026-09-17
> FR: FR-SOC-DET-002
> Epic: FR-SOC-DET

## User Intent

The product owner requires Social determinism as part of the FR-SOC-DET epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Social determinism is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-SOC-DET-002 contributes to the overall simulation capability by addressing: Social determinism.

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
2. The simulation runs without errors related to Social determinism
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-soc-det-002-spec.md` |
| ADR | `fr-soc-det-002-adr.md` |
| Research | `fr-soc-det-002-research.md` |
| Plan | `fr-soc-det-002-plan.md` |
| Implementing crate | `crates/civ-institutions/src/` |
