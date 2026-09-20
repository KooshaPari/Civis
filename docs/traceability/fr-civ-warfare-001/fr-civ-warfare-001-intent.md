# Intent: FR-CIV-WARFARE-001 -- War onset from diplomacy

> Date: 2026-09-19
> FR: FR-CIV-WARFARE-001
> Epic: FR-CIV-WARFARE

## User Intent

War must **emerge** from the diplomacy layer — rivalry, border friction,
and resource competition all push a polity pair's `Relation.standing`
downward; crossing the war-onset threshold announces a `WarState` to the
rest of the simulation. This is the "civilisation simulation, not war
game" property called out by the engagement model.

### What This FR Achieves

`crates/tactics/src/war_from_diplomacy.rs` provides:

- `WAR_STANDING_THRESHOLD = -60` — standing level below which a pair
  transitions to active war.
- `COMBAT_STANDING_DRAIN = -8` — standing drain per combat engagement.
- `RIVALRY_FRICTION_DRAIN = -3` — passive drain each tick from unresolved
  rivalry.
- `WarState { pair, onset_tick, ongoing }` — the announced transition.
- `check_war_onset(relation, current_tick) -> Option<WarState>` —
  fires exactly on the tick the standing first crosses the threshold.
- `apply_rivalry_friction(standing)` — passive drain helper.
- `apply_combat_drain(standing, engagements)` — per-engagement drain.
- `is_at_war(state) -> bool`.

## Acceptance Signal

- `cargo test -p tactics war_from_diplomacy` passes.
- No RNG; pure functions on the `Relation` snapshot.
- Standing `== WAR_STANDING_THRESHOLD` does **not** trigger war (strict
  inequality).

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/tactics/` |
| Module | `crates/tactics/src/war_from_diplomacy.rs:1` |
