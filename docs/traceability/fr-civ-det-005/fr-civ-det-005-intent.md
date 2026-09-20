# Intent: FR-CIV-DET-005 — Energy budget non-negativity

> Date: 2026-09-19
> FR: FR-CIV-DET-005
> Epic: FR-CIV-DET (Determinism)

## User Intent

The world `energy_budget_joules` field floors at zero: when consumption
exceeds the budget, the field saturates at zero rather than going
negative. Default state carries a positive budget. `step` correctly
subtracts consumption.

## Acceptance Signal

- `step(ws_with_50, 100)` → `energy_budget_joules == Fixed::ZERO`.
- `WorldState::default().energy_budget_joules > Fixed::ZERO`.
- `step` subtracts consumption from positive budgets.
- `crates/engine/tests/fr_fr_civ_det_005.rs` 3 tests pass.
- `// Covers: FR-CIV-DET-005` on the test module header.
