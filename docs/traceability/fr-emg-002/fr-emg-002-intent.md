# Intent: FR-EMG-002 -- Language emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-002
> Epic: FR-EMG

## User Intent

The language oracle must confirm that lexical divergence has
occurred across isolated population clusters: at least two clusters
must possess a non-empty `EvolvedLexicon` after tick > 0.

### What This FR Achieves

Verifies the phoneme-drift + co-location isolation loop is active
and that two populations have coined independent lexicons.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 2
thereafter.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the language oracle
  in `crates/engine/tests/fr_emg_oracle.rs:8`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/language.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/language.rs:19` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:8` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:53` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:211` |
| Implementing crate | `crates/emergence-oracle/src/` |
