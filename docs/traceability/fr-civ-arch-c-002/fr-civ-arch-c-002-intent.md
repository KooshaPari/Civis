# Intent: FR-CIV-ARCH-C-002 -- Building type unlock below min era is locked

> Date: 2026-09-20
> FR: FR-CIV-ARCH-C-002
> Epic: FR-CIV-ARCH-C

## User Intent

`building_type_unlocked(name, era)` must return `false` for any
building whose `min_era` is strictly greater than the supplied era,
covering Farm (era 0), Mine (era 1), Market (era 2), Temple (era 3),
and Barracks (era 4).

### What This FR Achieves

Establishes the boundary-condition half of the era-gate contract:
locked at era `min_era - 1`.

### Product Context

Pairs with FR-CIV-ARCH-C-003 to guarantee the unlock predicate is
exact across the transition point.

## Acceptance Signal

- Unit test `fr_arch_c002_building_type_below_min_era_is_locked`
  in `crates/build/src/tiers.rs:558` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/build/src/tiers.rs:558` |
| Implementing crate | `crates/build/src/` |
