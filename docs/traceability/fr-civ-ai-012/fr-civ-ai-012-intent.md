# Intent: FR-CIV-AI-012 -- Artificial intelligence

> Date: 2026-09-17
> FR: FR-CIV-AI-012
> Epic: FR-CIV-AI

## User Intent

The product owner requires Artificial intelligence as part of the FR-CIV-AI epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Artificial intelligence is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-AI-012 contributes to the overall simulation capability by addressing: Artificial intelligence.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/ai/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p ai` passes
2. The simulation runs without errors related to Artificial intelligence
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-ai-012-spec.md` |
| ADR | `fr-civ-ai-012-adr.md` |
| Research | `fr-civ-ai-012-research.md` |
| Plan | `fr-civ-ai-012-plan.md` |
| Implementing crate | `crates/ai/src/` |
