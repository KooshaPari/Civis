# Intent: FR-LANGUAGE-001 -- Per-faction emergent language state

> Date: 2026-09-19
> FR: FR-LANGUAGE-001
> Epic: FR-LANGUAGE

## User Intent

The product owner requires Per-faction emergent language state as part of the FR-LANGUAGE epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines `LanguageState` (a centroid signature +
drift rate + tick stamp) and the per-faction map (`faction_languages`) that
the `phase_language_drift` tick-phase seeds from dominant-settlement culture
language centroids, applies isolation-aware drift to, and prunes on extinct
faction removal.

Specifically:

- `crates/engine/src/engine.rs::LanguageState` (FR-LANGUAGE-001) — struct
  carrying `seed_signature`, drift rate, and seeded tick.
- `crates/engine/src/engine.rs::faction_languages` (FR-LANGUAGE-001) —
  `BTreeMap<u32, LanguageState>` per faction, driven by `phase_language_drift`.
- `crates/engine/src/engine/culture_phases.rs::phase_language_drift`
  (FR-CIV-LANG-001 / FR-LANGUAGE-001) — per-tick seeding, isolation drift,
  word-borrowing, and pruning.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-LANGUAGE-001 contributes to the overall simulation capability by
addressing: emergent language drift that names factions and feeds the
diplomacy intelligibility bonus.

## Acceptance Signal

### Definition of Done

- [x] `LanguageState` struct defined in `crates/engine/src/engine.rs`.
- [x] `faction_languages` BTreeMap maintained on Simulation.
- [x] `phase_language_drift` seeds / drifts / prunes per-tick.
- [x] `cargo build -p engine` passes.

### How We Know This FR Is Satisfied

1. Running a sim for many ticks causes per-faction language centroids to drift
   measurably when isolated and to converge when contact-connected.
2. Extinct factions are pruned from `faction_languages`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/engine.rs::LanguageState` |
| Code | `crates/engine/src/engine.rs::faction_languages` |
| Code | `crates/engine/src/engine/culture_phases.rs::phase_language_drift` |
| Implementing crate | `crates/engine/` |
