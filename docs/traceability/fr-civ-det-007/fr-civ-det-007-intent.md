# Intent: FR-CIV-DET-007 — Seeded RNG isolation

> Date: 2026-09-19
> FR: FR-CIV-DET-007
> Epic: FR-CIV-DET (Determinism)

## User Intent

`civ_engine::create_rng(seed)` returns a deterministic RNG. Two RNGs
constructed with the same seed must produce identical sequences;
different seeds must produce different sequences.

## Acceptance Signal

- `create_rng(42).gen::<u64>() == create_rng(42).gen::<u64>()`.
- `create_rng(1).gen::<u64>() != create_rng(999).gen::<u64>()`.
- `Simulation::with_seed(0)` starts in a deterministic state.
- `crates/engine/tests/fr_fr_civ_det_007.rs` 3 tests pass:
  `same_seed_same_rng_sequence`, `different_seeds_different_rng`,
  `simulation_seed_zero_deterministic`.
- `// Covers: FR-CIV-DET-007` on the test module header.
