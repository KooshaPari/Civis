# Intent: FR-CIV-PROTO-011 -- 3D protocol

> Date: 2026-09-17
> FR: FR-CIV-PROTO-011
> Epic: FR-CIV-PROTO

## User Intent

The product owner requires 3D protocol as part of the FR-CIV-PROTO epic for the Civis civilisation simulation.

### What This FR Achieves
This functional requirement ensures that 3D protocol is properly specified, implemented, and testable within the simulation engine.

### Product Context
Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates. FR-CIV-PROTO-011 contributes to the overall simulation capability by addressing: 3D protocol.

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
2. The simulation runs without errors related to 3D protocol
3. The feature is observable in the simulation output

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-proto-011-spec.md` |
| ADR | `fr-civ-proto-011-adr.md` |
| Research | `fr-civ-proto-011-research.md` |
| Plan | `fr-civ-proto-011-plan.md` |
| Implementing crate | `crates/protocol-3d/src/` |
