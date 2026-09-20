# Intent: FR-EMG-025 -- Migration pressure & trade-flow invariants

> Date: 2026-09-20
> FR: FR-EMG-025
> Epic: FR-EMG

## User Intent

Two oracles share the FR-EMG-025 contract:

- **Migration-pressure oracle**: confirms the famine cascade
  produces measurable migration_pressure per settlement — at least
  one settlement exists after tick ≥ 5.
- **Trade-flow oracle**: confirms settlement trade-flow mechanics
  respect the supply/demand invariants — flow direction matches sign
  of `(supply - demand)`, magnitude bounded by the smoothing factor,
  and exactly zero flow when supply == demand. All three invariant
  cases must pass.

### What This FR Achieves

Verifies that the famine cascade downstream (FR-CIV-FAMINE-001)
produces the migration-pressure channel that the agent layer reads,
and that the `settlement_trade_flow_from_supply_demand` helper in
`civ_economy` is invariant-safe.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0,
full coverage thereafter.

## Acceptance Signal

- Library unit tests in
  `crates/emergence-oracle/src/oracles/migration_pressure.rs:54`
  pass for both tick-0 and post-warmup regimes.

## Traceability

| Artifact | Path |
|----------|------|
| Test | `crates/emergence-oracle/src/oracles/migration_pressure.rs:54` |
| Code | `crates/emergence-oracle/src/oracles/migration_pressure.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/migration_pressure.rs:16` |
| Code | `crates/emergence-oracle/src/oracles/migration_pressure.rs:54` |
| Implementing crate | `crates/emergence-oracle/src/` |
