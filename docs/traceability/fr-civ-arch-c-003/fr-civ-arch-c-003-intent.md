# Intent: FR-CIV-ARCH-C-003 -- Building type unlock at min era is unlocked

> Date: 2026-09-20
> FR: FR-CIV-ARCH-C-003
> Epic: FR-CIV-ARCH-C

## User Intent

`building_type_unlocked(name, era)` must return `true` when the
supplied era equals the building's `min_era`, covering Farm (0),
Mine (1), Market (2), Temple (3), Barracks (4).

### What This FR Achieves

Establishes the inclusive boundary of the unlock predicate: the
transition point itself counts as unlocked.

### Product Context

Pairs with FR-CIV-ARCH-C-002 to guarantee the unlock predicate is
exact across the transition point.

## Acceptance Signal

- Unit test `fr_arch_c003_building_type_at_min_era_is_unlocked`
  in `crates/build/src/tiers.rs:576` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/build/src/tiers.rs:576` |
| Implementing crate | `crates/build/src/` |
