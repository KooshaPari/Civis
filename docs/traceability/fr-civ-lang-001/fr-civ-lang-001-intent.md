# Intent: FR-CIV-LANG-001 -- Language and naming

> Date: 2026-09-17
> FR: FR-CIV-LANG-001
> Epic: FR-CIV-LANG

## User Intent

The product owner requires Language and naming as part of the FR-CIV-LANG epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Language and naming is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-LANG-001 contributes to the overall simulation capability by addressing: Language and naming.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/i18n/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p i18n` passes
2. The simulation runs without errors related to Language and naming
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-lang-001-spec.md` |
| ADR | `fr-civ-lang-001-adr.md` |
| Research | `fr-civ-lang-001-research.md` |
| Plan | `fr-civ-lang-001-plan.md` |
| Implementing crate | `crates/i18n/src/` |
