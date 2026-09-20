# Intent: FR-CIV-CLIMATE-3 -- Drought likelihood peaks in summer (desert)

> Date: 2026-09-20
> FR: FR-CIV-CLIMATE-3
> Epic: FR-CIV-CLIMATE

## User Intent

For the Desert biome, summer must yield a higher
`drought_likelihood_fp` than spring, modelling the dry-season peak
of arid-climate stress.

### What This FR Achieves

Drives the disaster phase to fire drought events preferentially in
hot arid biomes during summer, so that desert settlements face
correctly-timed water scarcity.

### Product Context

Couples to the famine cascade (FR-CIV-FAMINE-001) and to the
disaster emergence oracle (FR-EMG-013).

## Acceptance Signal

- Unit test `drought_peaks_in_summer_desert` in
  `crates/planet/src/seasonal.rs:209` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/planet/src/seasonal.rs:209` |
| Implementing crate | `crates/planet/src/` |
