# Intent: FR-EMG-013 -- Disaster emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-013
> Epic: FR-EMG

## User Intent

The disaster oracle must confirm that disaster and climate-shock
emergence are active: at least one citizen and one building after
tick > 0, since disasters require both targets and environmental
exposure.

### What This FR Achieves

Verifies that natural-disaster and climate-shock subsystems have
inhabited, exposed landscape to act on.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter. Pairs with the seasonal modifiers in
`crates/planet/src/seasonal.rs` (FR-CIV-CLIMATE-3, -4).

## Acceptance Signal

- The disaster oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/disaster.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/disaster.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
