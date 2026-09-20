# Intent: FR-CIV-DET-004 — Tick counter monotonicity

> Date: 2026-09-19
> FR: FR-CIV-DET-004
> Epic: FR-CIV-DET (Determinism)

## User Intent

The simulation tick counter starts at zero, advances by exactly one on
each `tick()` or `step()` call, and never decreases. This is the clock the
hash chain keys off of — any non-monotonic regression breaks replay.

## Acceptance Signal

- `Simulation::with_seed(1).state.tick == 0`.
- Three consecutive `tick()` calls yield `state.tick == 1`, `2`, `3`.
- `crates/engine/tests/fr_fr_civ_det_004.rs` 3 tests pass:
  `tick_starts_at_zero`, `tick_advances_by_one`, `step_advances_tick_by_one`.
- `// Covers: FR-CIV-DET-004` on the test module header.
