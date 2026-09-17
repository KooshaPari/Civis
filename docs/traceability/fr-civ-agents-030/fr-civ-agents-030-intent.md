# Intent: FR-CIV-AGENTS-030 -- Agent systems

> Date: 2026-09-17
> FR: FR-CIV-AGENTS-030
> Epic: FR-CIV-AGENTS

## User Intent

The product owner requires Agent systems as part of the FR-CIV-AGENTS epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Agent systems is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-AGENTS-030 contributes to the overall simulation capability by addressing: Agent systems.

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
2. The simulation runs without errors related to Agent systems
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-agents-030-spec.md` |
| ADR | `fr-civ-agents-030-adr.md` |
| Research | `fr-civ-agents-030-research.md` |
| Plan | `fr-civ-agents-030-plan.md` |
| Implementing crate | `crates/ai/src/` |
