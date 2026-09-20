# Intent: FR-EMG-009 -- Migration emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-009
> Epic: FR-EMG

## User Intent

The migration oracle must confirm that creature migration and
movement dynamics are active: at least one citizen and one building
must exist after tick > 0 (settled, distributed population).

### What This FR Achieves

Verifies that agents are not stationary and that pathfinding /
settlement distribution is functional.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The migration oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/migration.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/migration.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
