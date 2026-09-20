# Intent: FR-CIV-BEVY-035 -- F3D0 binary round-trip across Frame3d kinds

> Date: 2026-09-19
> FR: FR-CIV-BEVY-035
> Epic: FR-CIV-BEVY

## User Intent

The product owner requires F3D0 binary round-trip across Frame3d kinds as part of the FR-CIV-BEVY epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that F3D0 binary round-trip across Frame3d kinds is properly specified, implemented, and testable within the simulation engine.

`clients/bevy-ref/src/lib.rs::parse_ws_payload` accepts both JSON text
frames and `F3D0`-prefixed binary frames. The
`parse_ws_payload_decodes_all_frame_kinds` test exercises every
`Frame3d` variant (`VoxelDelta`, `BuildingDiff`, `AgentAppearance`,
`CivilianState`, `FactionState`, `EventFeed`) through `encode_frame3d_binary`
→ `parse_ws_payload` and asserts byte-equal round-trip. The constant
`FRAME_BUNDLE_LEN = 7` mirror on the server side guarantees parity
between encoder and decoder.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with a
WebSocket / HTTP server. FR-CIV-BEVY-035 closes the wire-format
contract: every kind the server can emit is decodable in the Bevy
reference client (and replicable in Godot / Unreal via
`civ_protocol_3d`).

## Acceptance Signal

### Definition of Done

- [x] Implementation in `clients/bevy-ref/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] `parse_ws_payload_decodes_all_frame_kinds` covers every variant

### How We Know This FR Is Satisfied

1. `cargo test -p bevy-ref parse_ws_payload_decodes_all_frame_kinds` passes
2. The decoded value `==` the original value for every `Frame3d` kind
3. The decoder accepts legacy JSON text frames when no `F3D0` magic is present
4. Round-trip is deterministic (no time / RNG inputs)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-bevy-035-intent.md` |
| Source | `clients/bevy-ref/src/lib.rs:1973` |
| Test | `clients/bevy-ref/src/lib.rs:1973` |

<!-- Covers: FR-CIV-BEVY-035 -->
