# Intent: FR-CIV-GOV-100 — Social-mood inputs (food/housing/crime)

> FR: FR-CIV-GOV-100
> Epic: FR-CIV-GOV
> // Covers: FR-CIV-GOV-100

## Requirement

`phase_social_mood` consumes three per-settlement inputs that tests +
scenario loaders can set deterministically:

- `settlement_food_stocked` — units of food stocked; the phase derives
  `food_score = clamp(stocked / 200, MOOD_MIN, MOOD_MAX)`.
- `settlement_housing_capacity` — housing capacity; the phase derives
  `housing_score = clamp(2 * (capacity - population), MOOD_MIN, MOOD_MAX)`.
- `settlement_crime_pressure` — crime pressure; the phase derives
  `crime_score = max(0, MOOD_CRIME_BASE - 4 * pressure)`.

`MOOD_MIN = -200`, `MOOD_MAX = 200`, `MOOD_CRIME_BASE = 300` are the
saturating constants in `crates/engine/src/social_types.rs`.

## Source

- `crates/engine/src/engine.rs:870` — `settlement_food_stocked` field
  comments referencing FR-CIV-GOV-100.
- `crates/engine/src/engine.rs:875` — `settlement_housing_capacity`.
- `crates/engine/src/engine.rs:880` — `settlement_crime_pressure`.
- Test: `crates/engine/tests/fr_emergence_quality.rs:347`.

## Acceptance

- `cargo test -p civ-engine fr_emergence_quality` passes.
- A settlement with no entries in any of the three maps defaults each
  sub-score to `0`; the total mood is the sum of the three sub-scores
  plus Temple / Garrison bonuses, saturated to `[-200, 200]`.
