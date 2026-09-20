# Intent: FR-CIV-DIPLO-003-02 -- Leakage conservation: never goes negative

> Date: 2026-09-20
> FR: FR-CIV-DIPLO-003-02
> Epic: FR-CIV-DIPLO

## User Intent

Enforcement must never drive a pair's `total_leakage` below zero,
even when `max_leak_reduction` exceeds the current leakage; the
operation must saturate and emit `LeakReduced { amount, remaining: 0 }`.

### What This FR Achieves

Guarantees the shadow-network ledger is a non-negative counter, so
downstream legitimacy modifiers cannot be over-corrected.

### Product Context

Enforcing polities may overreach; FR-CIV-DIPLO-003-02 ensures the
ledger stays a valid `NonNegativeU64` and that `LegReduced` events
report the actual saturation amount.

## Acceptance Signal

- Unit tests `leakage_conservation_never_goes_negative` and
  `enforce_on_empty_pair_emits_no_event` in
  `crates/diplomacy/src/shadow_networks.rs:470` pass.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/diplomacy/src/shadow_networks.rs:470` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:472` |
| Code + test | `crates/diplomacy/src/shadow_networks.rs:509` |
| Implementing crate | `crates/diplomacy/src/` |
