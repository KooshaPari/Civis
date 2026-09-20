# Intent: FR-CIV-EMERGE-DASH-001 -- Emergence dashboard HUD panel

> Date: 2026-09-19
> FR: FR-CIV-EMERGE-DASH-001
> Epic: FR-CIV-EMERGE-DASH

## User Intent

The Bevy reference client (`clients/bevy-ref/`) needs an in-UI panel that
visualises the post-tick criticality readout so design/auditing users can
see whether the simulation is in the high-variance band the lore promises.
The panel is gated behind `bevy` + `egui` feature flags and toggled with
the `E` key, mirroring the diplomacy/faction panels in the same client.

### What This FR Achieves

`emergence_dashboard.rs` provides:

- `EmergenceDashboardState` (visible flag, `E` toggles) registered via
  `EmergenceDashboardPlugin`.
- An egui `EguiPrimaryContextPass` system (`draw_emergence_dashboard`)
  that reads `EmergenceHudData` polled every 10 s through `sim.emergence`
  (or in-process `SimState` in standalone mode).
- A compatibility alias `type EmergenceDashboardOpen = EmergenceDashboardState`
  for legacy callers.

## Acceptance Signal

- `cargo build -p bevy-ref --features "bevy egui"` succeeds.
- The panel renders under `E`, hides on second press.
- Polling cadence (10 s) is respected; no per-frame work in `Update`.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `clients/bevy-ref/` |
| Panel module | `clients/bevy-ref/src/emergence_dashboard.rs` |
