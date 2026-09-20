# Intent: FR-CIV-DIPLO-003-07 -- reset_tick clears per-tick state

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-07
> Epic: FR-CIV-DIPLO

## User Intent

`ShadowNetworkState::reset_tick` must clear all per-tick counters
(`active_pair_count`, `total_leakage`, and per-polity
`enforcement_intensity`) so that the next tick starts from zero.

### What This FR Achieves

Prevents stale pair aggregates from leaking into the next tick,
which would corrupt determinism and event ordering.

### Product Context

Companion to FR-CIV-DIPLO-003-06; the two halves of the reset
contract: counters zero, legitimacy persists.

## Acceptance Signal

- Unit test `reset_tick_clears_counters_preserves_legitimacy` in
  `crates/diplomacy/src/shadow_networks.rs:710` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:710` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:712` |
| Implementing crate | `crates/diplomacy/src/` |
