# Intent: FR-CIV-ARCH-C-004 -- Parcel kind era-unlock mirrors spec table

> Date: 2026-09-20
> FR: FR-CIV-ARCH-C-004
> Epic: FR-CIV-ARCH-C

## User Intent

`parcel_kind_min_era(kind)` and `parcel_kind_unlocked(kind, era)` must
match the spec table: Residential (0), Commercial (1), Industrial (2),
Civic (3). Civic in particular unlocks exactly at era 3, not earlier.

### What This FR Achieves

Locks the parcel-kind half of the era table so that the build loop
and the parcel allocator agree on what is buildable when.

### Product Context

Mirrors FR-CIV-ARCH-C-002 / C-003 at the parcel-kind abstraction so
both name-based and enum-based queries are answered consistently.

## Acceptance Signal

- Unit test `fr_arch_c004_parcel_kind_min_era_spec_table` in
  `crates/build/src/tiers.rs:586` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code + test | `crates/build/src/tiers.rs:586` |
| Implementing crate | `crates/build/src/` |
