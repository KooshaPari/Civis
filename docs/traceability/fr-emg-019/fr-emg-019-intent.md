# Intent: FR-EMG-019 -- Coastal settlement emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-019
> Epic: FR-EMG

## User Intent

The coastal-settlement oracle must confirm that coastal
infrastructure is emerging: at least one citizen and one building
after tick > 0, since real coastal settlement requires settled
infrastructure and population.

### What This FR Achieves

Verifies that the substrate for coastal development patterns exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The coastal-settlement oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/coastal_settlement.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/coastal_settlement.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
