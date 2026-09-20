# Intent: FR-EMG-020 -- River trade emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-020
> Epic: FR-EMG

## User Intent

The river-trade oracle must confirm that river-trade routes and
commerce are emerging: at least one citizen and one building after
tick > 0, since real river trade requires settled infrastructure
and population.

### What This FR Achieves

Verifies that the substrate for river-trade development patterns
exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter. Note: the source docstring labels this oracle as
"Coastal settlement" even though the FR is FR-EMG-020; both
classes share the citizen × building measurement.

## Acceptance Signal

- The river-trade oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/river_trade.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/river_trade.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
