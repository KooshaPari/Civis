# Intent: FR-CIV-TEST-001 -- N-series emergence coupling tests

> Date: 2026-09-19
> FR: FR-CIV-TEST-001
> Epic: FR-CIV-TEST

## User Intent

The product owner requires N-series emergence coupling tests as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines five coupling-path tests (N1/N2/N3/N4/N7)
that exercise previously-uncovered pathways:

- N1: market food-price response to settlement stock surplus (zero + boundary).
- N2: `CultureProfile::cultural_distance` API (identical + dissimilar).
- N3: settlement contact → diplomacy pair emission (≥2 factions) and the
  single-faction null case.
- N4: trade route resource transfer, overdraft guard, and zero-volume inertness.
- N7: sentience / awakening pipeline stability (long-run + tick-0 boundary).

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-001 contributes to the overall simulation capability by addressing:
external integration coverage for the N-series emergence coupling paths.

## Acceptance Signal

### Definition of Done

- [x] `crates/engine/tests/n_series_coverage.rs` exists with 11 tests covering N1/N2/N3/N4/N7.
- [x] `cargo test -p engine --test n_series_coverage` passes.
- [x] No regressions in existing FRs.

### How We Know This FR Is Satisfied

1. All 11 N-series tests pass.
2. Trade-route, market, culture-distance, diplomacy, and sentience pipelines are exercised externally.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/engine/tests/n_series_coverage.rs` |
| Implementing crate | `crates/engine/` |
