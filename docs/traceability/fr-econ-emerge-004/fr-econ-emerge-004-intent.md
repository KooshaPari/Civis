# Intent: FR-ECON-EMERGE-004 — Market shocks from disasters

> Date: 2026-09-19
> FR: FR-ECON-EMERGE-004
> Epic: FR-ECON-EMERGE

## User Intent

Market shocks translate external events (disasters, demand surges) into
price multipliers applied on top of the emergent supply/demand pricing.
The disaster kinds mirror `civ_engine::disasters::DisasterKind` without
depending on that crate; callers translate at the integration boundary.

## Acceptance Signal

- `crates/economy/src/shocks.rs:1` implements the price-multiplier shock
  layer.
- Each `DisasterKind` has a corresponding multiplier policy.
- Multipliers compose with emergent prices, never replace them.
- `// Covers: FR-ECON-EMERGE-004` on the module-level doc comment.
