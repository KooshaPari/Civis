# Intent: FR-EMG-022 -- Desert caravan emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-022
> Epic: FR-EMG

## User Intent

The desert-caravan oracle must confirm that desert-caravan routes
are emerging: at least one citizen and one building after tick > 0,
since real desert caravan requires settled infrastructure and
population.

### What This FR Achieves

Verifies that the substrate for desert-caravan development patterns
exists.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The desert-caravan oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/desert_caravan.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/desert_caravan.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
