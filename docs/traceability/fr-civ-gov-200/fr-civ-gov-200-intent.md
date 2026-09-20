# Intent: FR-CIV-GOV-200 — Stratification phase quantiles

> FR: FR-CIV-GOV-200
> Epic: FR-CIV-GOV
> // Covers: FR-CIV-GOV-200

## Requirement

The `phase_stratification` phase buckets each settlement's households
into Poor / Middle / Rich / Elite bands based on quintile cut-offs
(`q20`, `q40`, `q60`, `q80`). The phase emits `StratificationEvent`s
when households cross band boundaries and a `StratificationReport` at
the end of the tick.

The FR-CIV-GOV-200 family extends the FR-CIV-GOV-020 spec by adding
deterministic quintile thresholds and promotion / demotion accounting
that downstream phases (cohesion, unrest) consume.

## Source

- `crates/engine/src/engine/social_settlement_phases.rs:179` —
  `phase_stratification` implementation, header doc comments
  `// (FR-CIV-GOV-200 family)`.
- `crates/engine/src/social_types.rs` — `StratBand`, `StratQuantiles`,
  `StratificationEvent`, `StratificationReport`.

## Acceptance

- `cargo build -p civ-engine` succeeds with the phase_stratification
  body at `social_settlement_phases.rs:179` (i.e. no duplicate-symbol
  compile error).
- Quintile cut-offs match `n / 5`, `2n / 5`, `3n / 5`, `4n / 5` for
  `n >= 5`; smaller `n` degenerate-clamped to whichever band matches
  the index.
