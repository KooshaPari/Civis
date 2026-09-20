# Intent: FR-CIV-WARFARE-004 -- War legends from major battles

> Date: 2026-09-19
> FR: FR-CIV-WARFARE-004
> Epic: FR-CIV-WARFARE

## User Intent

The `legends` subsystem records culture-level narrative artefacts —
major battles and decisive victories need to promote into legend events
that downstream narrative generators and the Bevy `legends_ui` panel
can pick up.

### What This FR Achieves

`crates/tactics/src/war_legends.rs` provides:

- `LEGEND_BATTLE_MAGNITUDE_THRESHOLD = 0.4` — minimum battle magnitude
  required to emit a legend event.
- `DECISIVE_VICTORY_MAGNITUDE = 0.75` — magnitude used for events that
  qualify as decisive victories.
- `BattleSummary { tick, aggressor_faction, defender_faction, total_casualties, aggressor_casualties, region }`.
- `BattleSummary::magnitude() -> f32` — normalized to
  `total_casualties / 10_000` capped at `1.0`.
- `BattleSummary::is_decisive_victory()` — `true` when one side took
  ≥ 80% of total casualties.

The module emits `RawSimEvent`s into `legends::model` with the right
`EventKind` and `Role` so the legend subsystem can promote them.

## Acceptance Signal

- `cargo test -p tactics war_legends` passes.
- `magnitude(0)` → `0.0`; `magnitude(10_000)` → `1.0`; `magnitude(100_000)` → `1.0`.
- `is_decisive_victory()` returns `false` when `total_casualties == 0`.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/tactics/` |
| Module | `crates/tactics/src/war_legends.rs:1` |
