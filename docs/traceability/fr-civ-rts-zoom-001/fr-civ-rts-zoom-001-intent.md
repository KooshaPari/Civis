# Intent: FR-CIV-RTS-ZOOM-001 -- RTS zoom

> Date: 2026-09-17
> FR: FR-CIV-RTS-ZOOM-001
> Epic: FR-CIV-RTS-ZOOM

## User Intent

The product owner requires RTS zoom as part of the FR-CIV-RTS-ZOOM epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that RTS zoom is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-RTS-ZOOM-001 contributes to the overall simulation capability by addressing: RTS zoom.

## Acceptance Signal

### Definition of Done
- [ ] Implementation in `crates/protocol-3d/` compiles and passes all checks
- [ ] Unit tests pass for the new functionality
- [ ] Integration with the simulation tick system works correctly
- [ ] No regressions in existing FRs
- [ ] ADR is finalized and accepted
- [ ] This intent document is updated with final decisions

### How We Know This FR Is Satisfied
1. `cargo test -p protocol-3d` passes
2. The simulation runs without errors related to RTS zoom
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-rts-zoom-001-spec.md` |
| ADR | `fr-civ-rts-zoom-001-adr.md` |
| Research | `fr-civ-rts-zoom-001-research.md` |
| Plan | `fr-civ-rts-zoom-001-plan.md` |
| Implementing crate | `crates/protocol-3d/src/` |
