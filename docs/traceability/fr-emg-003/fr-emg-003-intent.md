# Intent: FR-EMG-003 -- Economy emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-003
> Epic: FR-EMG

## User Intent

The economy oracle must confirm that the market is clearing at
non-trivial prices: at least one good in
`SimulationSnapshot::market_prices` must have a positive clearing
price after tick > 0.

### What This FR Achieves

Verifies that supply/demand dynamics are at work; a dormant
allocator would leave every price at zero.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 1
thereafter.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the economy oracle
  in `crates/engine/tests/fr_emg_oracle.rs:9`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/economy.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/economy.rs:17` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:9` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:80` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:213` |
| Implementing crate | `crates/emergence-oracle/src/` |
