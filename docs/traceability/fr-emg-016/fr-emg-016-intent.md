# Intent: FR-EMG-016 -- Religious conflict / schism emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-016
> Epic: FR-EMG

## User Intent

The religious-conflict oracle must confirm that religious tensions
and schism are emerging: at least one citizen and one building
after tick > 0, since real religious conflict requires settled
infrastructure and population.

### What This FR Achieves

Verifies that the substrate for religious schism dynamics exists;
the religious-conflict phase has something to act on.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The religious-conflict oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/religious_conflict.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/religious_conflict.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
