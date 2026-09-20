# Intent: FR-CIV-TEST-006 -- civ-economy external coverage

> Date: 2026-09-19
> FR: FR-CIV-TEST-006
> Epic: FR-CIV-TEST

## User Intent

The product owner requires civ-economy external coverage as part of the FR-CIV-TEST epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines external coverage for four civ-economy
pub surfaces that lacked integration tests:

- `verify_ledger_conservation` — the `UnbalancedEntry` error variant.
- `step` — the no-change path (no tick-close entry when budget is unchanged).
- `MultiGoodMarket::place_order` — must match `place_bid`/`place_ask` output.
- `MultiGoodMarket::with_ttl` — custom TTL respected by `clear_all`.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-TEST-006 contributes to the overall simulation capability by
addressing: external validation of the ledger / market invariants.

## Acceptance Signal

### Definition of Done

- [x] `crates/economy/tests/economy_coverage.rs` exists with the four coverage tests.
- [x] `cargo test -p civ-economy --test economy_coverage` passes.

### How We Know This FR Is Satisfied

1. Unbalanced ledger entries are detected.
2. No-change `step` does not append a tick-close entry.
3. `place_order` produces identical trades to `place_bid`/`place_ask`.
4. Custom TTL keeps orders alive for the configured duration.

## Traceability

| Artifact | Path |
|----------|------|
| Tests | `crates/economy/tests/economy_coverage.rs` |
| Implementing crate | `crates/economy/` |
