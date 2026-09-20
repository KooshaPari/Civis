# Intent: FR-EMG-014 -- Mood emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-014
> Epic: FR-EMG

## User Intent

The mood oracle must confirm that social-mood and collective-
sentiment emergence are active: at least one citizen and one
building after tick > 0, since meaningful sentiment dynamics need
both agents and venues.

### What This FR Achieves

Verifies that the substrate for collective mood formation exists;
isolated agents cannot form collective mood.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- The mood oracle is auto-registered in
  `OracleRegistry::with_defaults`; coverage is enforced via the
  oracle_report CI gate.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/mood.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/mood.rs:17` |
| Implementing crate | `crates/emergence-oracle/src/` |
