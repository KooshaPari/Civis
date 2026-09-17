# Intent: FR-CIV-RESEARCH-033 -- Technology research

> Date: 2026-09-17
> FR: FR-CIV-RESEARCH-033
> Epic: FR-CIV-RESEARCH

## User Intent

The product owner requires Technology research as part of the FR-CIV-RESEARCH epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Technology research is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-RESEARCH-033 contributes to the overall simulation capability by addressing: Technology research.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/research/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p research` passes
2. The simulation runs without errors related to Technology research
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-research-033-spec.md` |
| ADR | `fr-civ-research-033-adr.md` |
| Research | `fr-civ-research-033-research.md` |
| Plan | `fr-civ-research-033-plan.md` |
| Implementing crate | `crates/research/src/` |
