# Intent: FR-CIV-CLIENT-013 — Civilization history sparklines

> Date: 2026-09-19
> FR: FR-CIV-CLIENT-013
> Epic: FR-CIV-CLIENT

## User Intent

The Bevy reference client exposes a civilization statistics history panel.
The `Y` key toggles it on/off. The panel samples population, entropy,
faction count, and power-law exponent every 10 ticks, keeps the most
recent 200 samples, and renders them as ASCII sparklines (8 levels).

## Acceptance Signal

- `clients/bevy-ref/src/civ_history.rs` defines `CivHistory` with
  `HISTORY_CAP == 200` and `SAMPLE_EVERY == 10`.
- `Y` keypress toggles visibility through the HUD state resource.
- ASCII sparkline renderer uses 8 glyph levels (e.g. `▁▂▃▄▅▆▇█`).
- `// Covers: FR-CIV-CLIENT-013` on the panel module header.
