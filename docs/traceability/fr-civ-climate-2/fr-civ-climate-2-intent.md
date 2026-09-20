# Intent: FR-CIV-CLIMATE-2 -- Food productivity peaks in autumn (plains)

> Date: 2026-09-20
> FR: FR-CIV-CLIMATE-2
> Epic: FR-CIV-CLIMATE

## User Intent

For the Plains biome, autumn must yield the highest
`food_productivity_fp` and winter the lowest, modelling the harvest
peak and dormant season.

### What This FR Achieves

Encodes the canonical agricultural calendar: spring sowing, summer
growth, autumn harvest, winter dormancy.

### Product Context

Drives the food economy tick; without the autumn peak, civilisations
cannot sustain populations through winter.

## Acceptance Signal

- Unit test `food_productivity_peaks_in_autumn_plains` in
  `crates/planet/src/seasonal.rs:193` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/planet/src/seasonal.rs:193` |
| Implementing crate | `crates/planet/src/` |
