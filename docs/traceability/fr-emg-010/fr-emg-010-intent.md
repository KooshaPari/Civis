# Intent: FR-EMG-010 -- Trade & epidemic emergence oracles

> Date: 2026-09-20
> FR: FR-EMG-010
> Epic: FR-EMG

## User Intent

Two oracles share the FR-EMG-010 contract:

- **Trade oracle**: confirms that trade and economic-flow emergence
  are active — at least one citizen and one building after tick > 0.
- **Epidemic oracle**: confirms disease/epidemic dynamics are active
  — at least one citizen and one building after tick > 0, since
  epidemic transmission requires both population spread and
  settlement infrastructure.

### What This FR Achieves

Verifies that the simulation has reached a state where trade and
epidemic subsystems have meaningful substrate to operate on.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The trade and epidemic oracles are auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/epidemic.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/epidemic.rs:17` |
| Code | `crates/emergence-oracle/src/oracles/trade.rs:1` |
| Implementing crate | `crates/emergence-oracle/src/` |
