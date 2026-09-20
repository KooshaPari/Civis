# Intent: FR-EMG-015 -- Stratification emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-015
> Epic: FR-EMG

## User Intent

The stratification oracle must confirm that social stratification
and class differentiation are emerging: at least one citizen and one
building after tick > 0, since real social hierarchy requires
settled infrastructure and population.

### What This FR Achieves

Verifies that the substrate for wealth and social-status hierarchies
exists; the stratification phase has something to act on.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The stratification oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/stratification.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/stratification.rs:18` |
| Implementing crate | `crates/emergence-oracle/src/` |
