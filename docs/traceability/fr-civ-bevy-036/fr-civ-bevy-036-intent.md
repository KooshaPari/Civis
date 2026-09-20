# Intent: FR-CIV-BEVY-036 -- Settings panel GPU readout

> Date: 2026-09-19
> FR: FR-CIV-BEVY-036
> Epic: FR-CIV-BEVY

## User Intent

The product owner requires Settings panel GPU readout as part of the FR-CIV-BEVY epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Settings panel GPU readout is properly specified, implemented, and testable within the simulation engine.

`clients/bevy-ref/src/menus.rs::format_gpu_settings_labels(caps)`
returns the read-only settings labels rendered in the Settings panel —
Backend, Est. VRAM, Ray tracing, DLSS, FSR — derived from the live
`GpuCapabilities` resource. Helper functions `format_gpu_capability_flag`,
`format_gpu_vram_label_mb` keep the formatting deterministic and
unit-testable so the panel is snapshot-stable across Bevy / Godot /
Unreal clients.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with a
WebSocket / HTTP server. FR-CIV-BEVY-036 contributes to the L2/L3
settings panel by guaranteeing the user can see which optional GPU
features (DLSS, FSR, ray tracing) are actually enabled for the running
session, removing guess-work from the render-quality question.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `clients/bevy-ref/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] Settings panel renders the five canonical labels

### How We Know This FR Is Satisfied

1. `cargo build -p bevy-ref` succeeds
2. `format_gpu_settings_labels` returns exactly five `(label, value)` rows
3. `format_gpu_vram_label_mb(0) == "Unknown"`, non-zero values formatted as
   `"<n> MB"`
4. `format_gpu_capability_flag(true|false)` returns `"Yes"` / `"No"`

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-bevy-036-intent.md` |
| Source | `clients/bevy-ref/src/menus.rs:4`, `:1348` |

<!-- Covers: FR-CIV-BEVY-036 -->
