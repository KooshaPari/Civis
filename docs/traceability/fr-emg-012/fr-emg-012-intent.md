# Intent: FR-EMG-012 -- Festival emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-012
> Epic: FR-EMG

## User Intent

The festival oracle must confirm that cultural-celebration emergence
is active: at least one citizen and one building after tick > 0,
since meaningful festivals require both agents and gathering venues.

### What This FR Achieves

Verifies the culture / festival substrate has both an audience
(citizens) and a venue (buildings) for celebration events.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The festival oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/festival.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/festival.rs:18` |
| Implementing crate | `crates/emergence-oracle/src/` |
