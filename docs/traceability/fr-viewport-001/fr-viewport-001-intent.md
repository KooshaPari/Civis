# Intent: FR-VIEWPORT-001 -- Minimap viewport indicator surface

> Date: 2026-09-19
> FR: FR-VIEWPORT-001
> Epic: FR-VIEWPORT

## User Intent

The product owner requires Minimap viewport indicator surface as part of the FR-VIEWPORT epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Minimap viewport indicator surface is properly specified, implemented, and testable within the simulation engine.

`crates/civis-cli/src/bin/three_d_quality.rs` lists the canonical
3D rendering kernel contracts that must be present for the Bevy
reference client to ship. One of those contracts is the minimap
viewport indicator: `clients/bevy-ref/src/minimap.rs` must contain the
identifier `viewport` (a public function or struct — e.g. the
`viewport` indicator component or function used by the camera HUD).
The 3D-quality gate asserts the file contains the needle and emits a
JSON receipt.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with
a WebSocket / HTTP server. FR-VIEWPORT-001 contributes to the 3D
quality gate by making the minimap viewport indicator a first-class
contract: if a refactor drops the `viewport` symbol, the gate fails
the build instead of silently shipping a HUD regression.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `clients/bevy-ref/` and `crates/civis-cli/` compiles
- [x] `minimap.rs` exports the `viewport` symbol (function or struct)
- [x] `just civis-3d-verify` (and `civis-3d-quality` CLI) reports the contract as present
- [x] No regressions in existing FRs

### How We Know This FR Is Satisfied

1. `cargo run -p civis-cli --bin civis-3d-quality` exits 0
2. JSON receipt `"contracts_missing_count": 0`
3. Second pass matches first (deterministic scan)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-viewport-001-intent.md` |
| Source | `crates/civis-cli/src/bin/three_d_quality.rs:54` |

<!-- Covers: FR-VIEWPORT-001 -->
