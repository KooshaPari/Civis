# Intent: FR-CIV-BUILD-004 -- Building tiers and construction

> Date: 2026-09-17
> FR: FR-CIV-BUILD-004
> Epic: FR-CIV-BUILD

## User Intent

The product owner requires Building tiers and construction as part of the FR-CIV-BUILD epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that Building tiers and construction is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-BUILD-004 contributes to the overall simulation capability by addressing: Building tiers and construction.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/physics-substrate/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p physics-substrate` passes
2. The simulation runs without errors related to Building tiers and construction
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-build-004-spec.md` |
| ADR | `fr-civ-build-004-adr.md` |
| Research | `fr-civ-build-004-research.md` |
| Plan | `fr-civ-build-004-plan.md` |
| Implementing crate | `crates/physics-substrate/src/` |
