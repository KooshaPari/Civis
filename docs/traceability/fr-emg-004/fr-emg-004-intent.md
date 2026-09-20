# Intent: FR-EMG-004 -- Legends emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-004
> Epic: FR-EMG

## User Intent

The legends oracle must confirm that at least one legend entity has
been promoted and recorded in the saga graph: `legends_query("status")`
must return ≥ 1 node after tick > 0.

### What This FR Achieves

Verifies that the `legends` ingest phase has processed at least one
in-world event (birth, death, sentience, battle, founding) and
promoted it to the saga graph.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the legends oracle
  in `crates/engine/tests/fr_emg_oracle.rs:10`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/legends.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/legends.rs:18` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:10` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:92` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:216` |
| Implementing crate | `crates/emergence-oracle/src/` |
