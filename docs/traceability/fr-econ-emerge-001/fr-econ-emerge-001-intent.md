# Intent: FR-ECON-EMERGE-001 — Emergent per-cluster per-good prices

> Date: 2026-09-19
> FR: FR-ECON-EMERGE-001
> Epic: FR-ECON-EMERGE

## User Intent

Prices in the economy crate emerge from local supply/demand ratios rather
than from a fixed oracle. For each `ClusterId × Good`, the price is
derived from the ratio of demand to supply. There is no absolute price
oracle; every price is relative and emergent.

## Acceptance Signal

- `crates/economy/src/prices.rs:1` exposes the per-cluster per-good price
  computation.
- `PriceState` carries the relative pricing for every `ClusterId × Good`
  pair, keyed by `ClusterId` (u64).
- `// Covers: FR-ECON-EMERGE-001` on the module-level doc comment.
