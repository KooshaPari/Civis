# Intent: FR-CIV-ARCH-C-001 -- Era-gated demand signal suppression

> Date: 2026-09-20
> FR: FR-CIV-ARCH-C-001
> Epic: FR-CIV-ARCH-C

## User Intent

Era-gated building unlocks must also suppress the underlying demand
signals so that the architecture phase cannot emit demand for
building types the polity has not yet unlocked.

### What This FR Achieves

`era_gated_demand_signals(full, era)` zeroes the `commercial`,
`industrial`, and `civic` channels at era 0; releases `industrial` at
era 2 and `civic` at era 3; leaves `residential` active throughout.

### Product Context

Civis is a Rust-based civilisation simulation. FR-CIV-ARCH-C-001
contributes by preventing the build loop from requesting locked
building types even when raw demand would otherwise justify them.

## Acceptance Signal

- Unit test `fr_arch_c001_era_gate_suppresses_locked_demand_channels`
  in `crates/build/src/tiers.rs:530` passes.
- The function is invoked by the architecture tick phase for every
  cluster before demand dispatch.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/build/src/tiers.rs:530` |
| Implementing crate | `crates/build/src/` |
