# Intent: FR-CIV-VERIFY-007 -- Verification harness

> Date: 2026-09-17
> FR: FR-CIV-VERIFY-007
> Epic: FR-CIV-VERIFY

## User Intent

The product owner requires Verification harness as part of the FR-CIV-VERIFY epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Verification harness is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-VERIFY-007 contributes to the overall simulation capability by addressing: Verification harness.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/build/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p build` passes
2. The simulation runs without errors related to Verification harness
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-verify-007-spec.md` |
| ADR | `fr-civ-verify-007-adr.md` |
| Research | `fr-civ-verify-007-research.md` |
| Plan | `fr-civ-verify-007-plan.md` |
| Implementing crate | `crates/build/src/` |
