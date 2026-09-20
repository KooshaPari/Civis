# Intent: FR-CIV-BEVY-028 -- F3D0 binary tick-broadcast bundle

> Date: 2026-09-19
> FR: FR-CIV-BEVY-028
> Epic: FR-CIV-BEVY

## User Intent

The product owner requires F3D0 binary tick-broadcast bundle as part of the FR-CIV-BEVY epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that F3D0 binary tick-broadcast bundle is properly specified, implemented, and testable within the simulation engine.

`crates/server/src/ws_bridge.rs` exports `pub const FRAME_BUNDLE_LEN: usize = 7`,
the number of distinct `Frame3d` variants that the 10 Hz tick loop
emits per simulation tick when `TickBroadcastFormat::Binary` or `Both`
is selected. WS clients attach with `?tick_format=binary` to receive
the `F3D0`-prefixed bundle (frame kind tags plus payload) instead of
the legacy JSON text frames.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with a
WebSocket / HTTP server (`civ-server`). FR-CIV-BEVY-028 contributes to
the 3D client attach matrix by guaranteeing that every binary tick
contains the expected number of decodable `Frame3d` kinds so the Bevy
reference client and Godot / Unreal clients all stay in lockstep.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `crates/server/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] WS smoke tests assert six-valid-frame bundle for `Both` / `Binary` formats
      (`crates/server/tests/ws_smoke.rs:1513`, `:1522`)

### How We Know This FR Is Satisfied

1. `cargo test -p civ-server ws_sim_command_tick_broadcasts_f3d0` passes
2. `FRAME_BUNDLE_LEN` constant matches the live `Frame3d` variant count
3. The binary tick payload decodes through `parse_ws_payload` for every kind

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-bevy-028-intent.md` |
| Source | `crates/server/src/ws_bridge.rs:57` |
| Tests | `crates/server/tests/ws_smoke.rs:1513`, `:1522` |

<!-- Covers: FR-CIV-BEVY-028 -->
