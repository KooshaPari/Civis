# Intent: FR-EMG-017 -- Expansion / stratification emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-017
> Epic: FR-EMG

## User Intent

The expansion oracle must confirm that polity expansion dynamics
are emerging: at least one citizen and one building after tick > 0,
since real expansion requires settled infrastructure and population.

### What This FR Achieves

Verifies that the substrate for polity-expansion dynamics exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter. Note: the source docstring labels this oracle as the
"stratification" oracle even though the FR is FR-EMG-017; both
classes share the citizen × building measurement.

## Acceptance Signal

- The expansion oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/expansion.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/expansion.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
