# Intent: FR-CIV-GAME-002 -- God-mode panel + scenario objectives

> Date: 2026-09-19
> FR: FR-CIV-GAME-002
> Epic: FR-CIV-GAME

## User Intent

The product owner requires God-mode panel + scenario objectives as part of the FR-CIV-GAME epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the God-mode intervention panel (toggled with
`G`) and the engine-side gameplay loop that classifies victory/defeat, tracks
per-faction progress, and exposes deterministic scenario-objective resolution.

Specifically:

- `clients/bevy-ref/src/god_panel.rs` (FR-CIV-GAME-002) — God mode panel that
  lets the player send `GodActionRequest`s (selected action, magnitude,
  target_x/y, target faction) to the server via `sim.god_action`.
- `crates/engine/src/gameplay.rs` (FR-CIV-GAME-002) — gameplay loop with
  `VictoryKind` enum (Domination, Cultural, Economic, Scientific, Diplomatic),
  per-faction progress, composite score, and `compute_gameplay_state` that
  resolves victory/defeat via the same engine conditions as `GameOutcome`.
- `objectives_progression_over_ticks` test confirms scenario objectives
  deterministically expire to a `Defeat` if `tick >= tick_limit` while
  victory threshold stays unmet.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-GAME-002 contributes to the overall simulation capability by addressing:
the player-facing God mode and the engine gameplay loop that classifies when
the simulation reaches a terminal state.

## Acceptance Signal

### Definition of Done

- [x] `crates/engine/src/gameplay.rs` defines `VictoryKind`, `compute_gameplay_state`, scenario objectives.
- [x] `crates/engine/src/gameplay.rs::objectives_progression_over_ticks` test passes.
- [x] `clients/bevy-ref/src/god_panel.rs` builds and renders the God mode panel.
- [x] `cargo build -p engine -p bevy-ref` passes.

### How We Know This FR Is Satisfied

1. `cargo test -p engine gameplay::objectives_progression_over_ticks` passes.
2. Pressing G in-game shows the God mode panel and lets the player submit a God action.
3. A scenario objective whose threshold is unmet past `tick_limit` returns `Defeat(...)`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `clients/bevy-ref/src/god_panel.rs` |
| Code | `crates/engine/src/gameplay.rs` |
| Tests | `crates/engine/src/gameplay.rs::objectives_progression_over_ticks` |
| Implementing crate | `crates/engine/`, `clients/bevy-ref/` |
