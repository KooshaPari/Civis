# Intent: FR-EMG-008 -- Creature emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-008
> Epic: FR-EMG

## User Intent

The creature oracle must confirm that the genetics–speciation loop
has differentiated at least two species lineages: ≥ 2 distinct
cluster culture profiles after tick > 0.

### What This FR Achieves

Falls back to military-units + cluster-cultures check when
sentience events have not yet fired (early-game); confirms that
genetic drift produces meaningfully diverse lineages.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 2
thereafter. Currently one of the two oracles not passing at the 300
tick baseline.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the creature oracle
  in `crates/engine/tests/fr_emg_oracle.rs:14`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/bin/oracle_report.rs:18` |
| Code | `crates/emergence-oracle/src/oracles/creature.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/creature.rs:24` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:14` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:26` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:175` |
| Implementing crate | `crates/emergence-oracle/src/` |
