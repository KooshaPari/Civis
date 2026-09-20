# Intent: FR-CIV-DIPLOMACY-004 — Diplomacy stance opinion model

> Date: 2026-09-20
> FR: FR-CIV-DIPLOMACY-004
> Epic: FR-CIV-DIPLOMACY

## What This FR Captures

The per-pair opinion-vector diplomacy model in
`crates/diplomacy/src/stance.rs`. Each `(PolityId, PolityId)` pair
carries a `DiplomacyStance { trust, fear, respect }` triple, all
clamped f32 in well-defined ranges. The coarse
`EmergentStance {Rival, Neutral, Ally}` (FR-CIV-DIPLOMACY) is
derived from the opinion vector, but the opinion vector itself
provides the basis for concession bias, alliance eligibility,
and AI decisions.

## User Intent

The legacy coarse stance (one of three buckets) is not expressive
enough for narrative-driven diplomacy: two polities might both be
"Ally" in the coarse model but for very different reasons (mutual
trust vs shared existential threat). The opinion vector captures
that nuance and lets downstream systems (concessions, treaties,
trade bias) reason about *why* a pair is where they are.

## Acceptance Signal

- `DiplomacyStance::default()` returns all-zero axes.
- `coarse_stance()` partitions into Ally/Neutral/Rival per the
  documented thresholds.
- `concession_bias()` returns a clamped `[-1, 1]` composite.
- Stance decays toward zero per tick (linear) and is updated by
  `InteractionEvent`s.
- Determinism: f32 clamped, BTreeMap iteration sorted, no RNG.

## Implementing Code

- `crates/diplomacy/src/stance.rs:1` — module header w/ FR-CIV-DIPLOMACY-004 ref.
- `crates/engine/src/engine.rs:440` — `WorldState` declares the
  durable `stance_engine` field.
- `crates/engine/src/engine.rs:1003` — `Simulation` holds the
  live `stance_engine` and applies decay each tick.

## Test Coverage

- Tests live in `crates/diplomacy/src/stance.rs` (and are
  exercised by `crates/engine/tests/diplomacy_flow.rs`).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-diplomacy-004-intent.md` |
| Implementing crate | `crates/diplomacy/src/stance.rs` |