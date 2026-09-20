# Intent: FR-EMG-006 -- Psyche emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-006
> Epic: FR-EMG

## User Intent

The psyche oracle must confirm that cluster belief centroids have
diverged from the genome baseline: at least one cluster must have a
belief centroid with a non-zero component after tick > 0.

### What This FR Achieves

Verifies that OCEAN trait states are being written and accumulated
via `emergence_accrue_cluster_beliefs`.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the psyche oracle
  in `crates/engine/tests/fr_emg_oracle.rs:12`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/psyche.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/psyche.rs:19` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:12` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:132` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:222` |
| Implementing crate | `crates/emergence-oracle/src/` |
