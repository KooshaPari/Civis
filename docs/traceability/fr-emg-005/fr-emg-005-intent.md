# Intent: FR-EMG-005 -- Diplomacy emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-005
> Epic: FR-EMG

## User Intent

The diplomacy oracle must confirm that the diplomacy phase ran and
resolved at least one faction-pair outcome: ≥ 1 `DiplomacyEvent`
(TradeAgreement, Conflict, or Peace) must appear after tick > 0.

### What This FR Achieves

Verifies that the diplomacy phase actually evaluates faction-pair
signals rather than leaving every pair in neutral inertia.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter. Currently one of the two oracles not passing at the 300
tick baseline.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the diplomacy oracle
  in `crates/engine/tests/fr_emg_oracle.rs:11`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/bin/oracle_report.rs:17` |
| Code | `crates/emergence-oracle/src/oracles/diplomacy.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/diplomacy.rs:19` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:11` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:26` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:103` |
| Implementing crate | `crates/emergence-oracle/src/` |
