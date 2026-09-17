# Intent: FR-CIV-LAWS-000 -- Legal system

> Date: 2026-09-17
> FR: FR-CIV-LAWS-000
> Epic: FR-CIV-LAWS

## User Intent

The product owner requires Legal system as part of the FR-CIV-LAWS epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Legal system is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-LAWS-000 contributes to the overall simulation capability by addressing: Legal system.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/laws/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p laws` passes
2. The simulation runs without errors related to Legal system
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-laws-000-spec.md` |
| ADR | `fr-civ-laws-000-adr.md` |
| Research | `fr-civ-laws-000-research.md` |
| Plan | `fr-civ-laws-000-plan.md` |
| Implementing crate | `crates/laws/src/` |
