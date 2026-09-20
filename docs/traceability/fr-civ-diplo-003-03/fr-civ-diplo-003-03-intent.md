# Intent: FR-CIV-DIPLO-003-03 -- Enforcement intensity & overreach detection

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-03
> Epic: FR-CIV-DIPLO

## User Intent

Each polity accumulates `enforcement_intensity` per pair enforced.
When the per-polity count exceeds `overreach_threshold`, an
`OverreachDetected` event fires and `legitimacy_modifier` is
decremented by `overreach_legitimacy_delta`.

### What This FR Achieves

Modelling of state overreach: polities that lean too hard on
shadow-network enforcement pay a legitimacy cost.

### Product Context

Drives the legitimacy pipeline that flows into the diplomacy
stance resolution. Per-polity counters prevent one state's
overreach from punishing another.

## Acceptance Signal

- Unit tests `enforcement_overreach_detection` and
  `enforcement_intensity_is_per_polity` in
  `crates/diplomacy/src/shadow_networks.rs:520` pass.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:520` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:522` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:560` |
| Implementing crate | `crates/diplomacy/src/` |
