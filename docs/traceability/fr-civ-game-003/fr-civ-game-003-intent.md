# Intent: FR-CIV-GAME-003 -- Era evaluation + HUD

> Date: 2026-09-19
> FR: FR-CIV-GAME-003
> Epic: FR-CIV-GAME

## User Intent

The product owner requires Era evaluation + HUD as part of the FR-CIV-GAME epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the engine-side `CivEra::evaluate` logic
that derives a civilization era (`Stone → Industrial`) from a faction's
population, unlocked tech level, and economic surplus, and the Bevy HUD
(`era_hud.rs`) that surfaces era advances as event-feed toasts.

Specifically:

- `crates/engine/src/era.rs::CivEra::evaluate` (FR-CIV-GAME-003) — derives
  era on demand from `Simulation` state. No persistent field; the consumer
  compares successive values to detect advances.
- `clients/bevy-ref/src/era_hud.rs::EraHudPlugin` (FR-CIV-GAME-003) — polls
  `sim.tech_state`, derives the era string from snapshot (Prehistoric → Modern),
  and pushes a `FeedKind::System` toast when the era advances.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-GAME-003 contributes to the overall simulation capability by addressing:
civilization-era progression visible to the player via the HUD and event feed.

## Acceptance Signal

### Definition of Done

- [x] `crates/engine/src/era.rs::CivEra` derives era from population + tech + surplus.
- [x] `clients/bevy-ref/src/era_hud.rs` polls `sim.tech_state` and surfaces era.
- [x] `cargo build -p engine -p bevy-ref` passes.

### How We Know This FR Is Satisfied

1. `CivEra::evaluate` returns a strictly ordered era (no regressions).
2. Era advances are pushed to the event feed as system toasts.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/era.rs::CivEra` |
| Code | `clients/bevy-ref/src/era_hud.rs` |
| Implementing crate | `crates/engine/`, `clients/bevy-ref/` |
