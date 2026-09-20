# Intent: FR-CIV-CLIMATE-4 -- Flood likelihood peaks in spring (wetland)

> Date: 2026-09-20
> FR: FR-CIV-CLIMATE-4
> Epic: FR-CIV-CLIMATE

## User Intent

For the Wetland biome, spring must yield a higher
`flood_likelihood_fp` than summer, modelling the wet-season peak of
fluvial flooding in saturated biomes.

### What This FR Achieves

Drives the disaster phase to fire flood events preferentially in
wet biomes during spring, so that wetland settlements face
correctly-timed inundation.

### Product Context

Pairs with FR-CIV-CLIMATE-3 (summer drought for deserts) to give the
disaster emergence oracle (FR-EMG-013) biome- and season-aware
behaviour.

## Acceptance Signal

- Unit test `floods_peak_in_spring_wetland` in
  `crates/planet/src/seasonal.rs:218` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/planet/src/seasonal.rs:218` |
| Implementing crate | `crates/planet/src/` |
