# Intent: FR-EMG-001 -- Religion emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-001
> Epic: FR-EMG

## User Intent

The religion oracle must confirm that the disaster → faith belief
loop is alive: after tick > 0, either `sim.belief() > 0` (raw belief
currency accumulated) or `sim.has_religious_patron()` (shared
veneration crystallised from saga promotions).

### What This FR Achieves

Establishes the religion pillar of the emergence CI gate: a
simulation that never produces belief is broken.

### Product Context

Wired into `oracle_report` CI baseline; threshold is 0 at tick 0
(warmup) and 1 thereafter.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the religion oracle
  in `crates/engine/tests/fr_emg_oracle.rs:7`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/lib.rs:15` |
| Code | `crates/emergence-oracle/src/oracles/religion.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/religion.rs:17` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:7` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:40` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:208` |
| Implementing crate | `crates/emergence-oracle/src/` |
