# Intent: FR-CIV-CLIMATE-1 -- Seasonal modifiers are deterministic

> Date: 2026-09-20
> FR: FR-CIV-CLIMATE-1
> Epic: FR-CIV-CLIMATE

## User Intent

`seasonal_modifiers(season, biome)` must return the same
`SeasonalModifiers` struct for identical inputs across repeated
calls, so that climate effects are reproducible in replays.

### What This FR Achieves

Establishes the determinism contract for season × biome lookups;
every other FR-CIV-CLIMATE-N test relies on this property.

### Product Context

Civis requires bit-perfect replay; the climate path is one of the
first lookup tables hit each tick, so determinism here is a
precondition for downstream FR coverage.

## Acceptance Signal

- Unit test `modifiers_are_deterministic_for_same_inputs` in
  `crates/planet/src/seasonal.rs:181` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/planet/src/seasonal.rs:181` |
| Implementing crate | `crates/planet/src/` |
