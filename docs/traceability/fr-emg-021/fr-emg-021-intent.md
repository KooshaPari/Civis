# Intent: FR-EMG-021 -- Mountain pass emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-021
> Epic: FR-EMG

## User Intent

The mountain-pass oracle must confirm that mountain-pass
infrastructure is emerging: at least one citizen and one building
after tick > 0, since real mountain-pass routes require settled
infrastructure and population.

### What This FR Achieves

Verifies that the substrate for mountain-pass development patterns
exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The mountain-pass oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/mountain_pass.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/mountain_pass.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
