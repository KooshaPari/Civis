# Intent: FR-CIV-DET-003 — Fixed-point arithmetic is exact

> Date: 2026-09-19
> FR: FR-CIV-DET-003
> Epic: FR-CIV-DET (Determinism)

## User Intent

The `Fixed` fixed-point type used by the engine must produce bit-exact
results for addition and subtraction. Zero and one are defined as named
constants. This is the cornerstone of cross-platform determinism — no
`f32`/`f64` rounding noise can leak into per-tick state mutation.

## Acceptance Signal

- `civ_engine::Fixed::ZERO` and `Fixed::from_num(0)` are equal.
- `Fixed::from_num(100) + Fixed::from_num(200) == Fixed::from_num(300)`.
- `Fixed::from_num(500) - Fixed::from_num(200) == Fixed::from_num(300)`.
- `crates/engine/tests/fr_fr_civ_det_003.rs` 3+ tests pass.
- `// Covers: FR-CIV-DET-003` on the test module header.
