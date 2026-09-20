# Intent: FR-CIV-GOV-020 — Stratification types and phase

> FR: FR-CIV-GOV-020
> Epic: FR-CIV-GOV
> // Covers: FR-CIV-GOV-020

## Requirement

`phase_stratification` computes per-settlement household wealth quantiles
(Poor / Middle / Rich / Elite), derives a Gini coefficient, and emits
`StratificationEvent`s for promotion / demotion / unchanged households.
The types are defined in `crates/engine/src/social_types.rs`.

## Source

- `crates/engine/src/social_types.rs:5` — module doc listing
  `FR-CIV-GOV-020` as a covered FR.
- `crates/engine/src/social_types.rs:69` — `StratBand`, `StratQuantiles`,
  `StratificationEvent`, `StratificationReport`, `compute_gini`.
- `crates/engine/src/engine/social_settlement_phases.rs:180` —
  `phase_stratification`.
- Test: `crates/engine/tests/fr_civ_gov_stratification.rs`.

## Acceptance

- `cargo test -p civ-engine fr_civ_gov_stratification` passes.
- Gini is clamped to `[0.0, 1.0]`; non-finite results coerce to `0.0`.
- Promotion / demotion events fire only on band transitions, never on
  within-band score drift.
