# Intent: FR-EMG-007 -- Architecture emergence oracle

> Date: 2026-09-20
> FR: FR-EMG-007
> Epic: FR-EMG

## User Intent

The architecture oracle must confirm that the building graph has
been seeded with multiple distinct structure types: total building
count must be ≥ 3 after tick > 0 (CityCenter + at least one Farm +
one other type).

### What This FR Achieves

Verifies that era-gating (FR-CIV-ARCH-C-001..004) and biome-driven
style variation (FR-CIV-ARCH-001 / FR-CIV-ARCH-002) produce a
type-diverse building graph.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0, ≥ 3
thereafter. The 300-tick CI integration test asserts it.

## Acceptance Signal

- Integration test `fr_emg_oracle` exercises the architecture
  oracle in `crates/engine/tests/fr_emg_oracle.rs:13`.
- Library unit test asserts post-300-tick pass at
  `crates/emergence-oracle/src/lib.rs:158`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/lib.rs:158` |
| Code | `crates/emergence-oracle/src/lib.rs:159` |
| Code | `crates/emergence-oracle/src/oracles/architecture.rs:1` |
| Test | `crates/engine/tests/fr_emg_oracle.rs:13` |
| Implementing crate | `crates/emergence-oracle/src/` |
