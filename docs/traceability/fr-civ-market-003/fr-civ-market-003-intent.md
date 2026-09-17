# Intent: FR-CIV-MARKET-003 -- Market dynamics

> Date: 2026-09-17
> FR: FR-CIV-MARKET-003
> Epic: FR-CIV-MARKET

## User Intent

The product owner requires Market dynamics as part of the FR-CIV-MARKET epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Market dynamics is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-MARKET-003 contributes to the overall simulation capability by addressing: Market dynamics.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/economy/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p economy` passes
2. The simulation runs without errors related to Market dynamics
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-market-003-spec.md` |
| ADR | `fr-civ-market-003-adr.md` |
| Research | `fr-civ-market-003-research.md` |
| Plan | `fr-civ-market-003-plan.md` |
| Implementing crate | `crates/economy/src/` |
