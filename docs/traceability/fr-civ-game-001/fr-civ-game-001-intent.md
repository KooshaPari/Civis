# Intent: FR-CIV-GAME-001 -- Gameplay HUD + outcome overlay

> Date: 2026-09-19
> FR: FR-CIV-GAME-001
> Epic: FR-CIV-GAME

## User Intent

The product owner requires Gameplay HUD + outcome overlay as part of the FR-CIV-GAME epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the in-game HUD that surfaces the
faction leaderboard, victory progress, and a full-screen modal for terminal
outcomes. The clients/bevy-ref HUD reads:

- Live faction data from `LiveStreamScene` (composite treasury score, win condition progress).
- `OutcomeOverlayState` to render the modal when `sim.outcome` returns a non-`Ongoing` result.
- `OutcomeProgressHud` / `OutcomeHudData` resources populated by polling
  `sim.outcome` every 30 s via the WsClient background thread.

`gameplay_hud.rs` (toggled with `F9`) shows faction leaderboard, victory progress,
and the outcome banner. `outcome_overlay.rs` shows the modal overlay with a
`[New Game]` button that sends `sim.reset`.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-GAME-001 contributes to the overall simulation capability by addressing:
client-side rendering of faction standings and game-end conditions on top of the
JSON-RPC `sim.outcome` polling channel.

## Acceptance Signal

### Definition of Done

- [x] `clients/bevy-ref/src/gameplay_hud.rs` renders faction leaderboard, victory progress, outcome banner.
- [x] `clients/bevy-ref/src/outcome_overlay.rs` polls `sim.outcome` and renders modal.
- [x] `clients/bevy-ref/src/lib.rs::OutcomeHudData` resource holds outcome data from `sim.outcome` polling.
- [x] `cargo build -p bevy-ref` passes.

### How We Know This FR Is Satisfied

1. Pressing F9 in-game shows the gameplay HUD with live faction standings.
2. When `sim.outcome` returns a terminal result, the modal overlay appears.
3. The `[New Game]` button sends `sim.reset`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `clients/bevy-ref/src/gameplay_hud.rs` |
| Code | `clients/bevy-ref/src/outcome_overlay.rs` |
| Code | `clients/bevy-ref/src/lib.rs::OutcomeHudData` |
| Implementing crate | `clients/bevy-ref/` |
