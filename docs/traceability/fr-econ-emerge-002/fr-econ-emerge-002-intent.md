# Intent: FR-ECON-EMERGE-002 — Trade flows on price differentials

> Date: 2026-09-19
> FR: FR-ECON-EMERGE-002
> Epic: FR-ECON-EMERGE

## User Intent

Trade flows emerge from price differentials between clusters. Surplus
goods flow from low-price (high-supply) clusters to high-price
(scarcity) clusters; the flow volume is proportional to the price
differential. No central trade-router exists — flow is emergent.

## Acceptance Signal

- `crates/economy/src/trade.rs:1` implements the differential-driven
  flow.
- Flow volume scales with the price differential between source and sink
  clusters for each `Good`.
- `// Covers: FR-ECON-EMERGE-002` on the module-level doc comment.
