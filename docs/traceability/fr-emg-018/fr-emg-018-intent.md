# Intent: FR-EMG-018 -- Migration flow emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-018
> Epic: FR-EMG

## User Intent

The migration-flow oracle must confirm that migration flow and
citizen mobility are emerging: at least one citizen and one building
after tick > 0, since real migration requires settled
infrastructure and population.

### What This FR Achieves

Verifies that the substrate for citizen mobility dynamics exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The migration-flow oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/migration_flow.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/migration_flow.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
