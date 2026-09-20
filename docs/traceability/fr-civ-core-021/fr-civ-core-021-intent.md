# Intent: FR-CIV-CORE-021 -- Primitive tile-set id fallback

> Date: 2026-09-19
> FR: FR-CIV-CORE-021
> Epic: FR-CIV-CORE

## User Intent

When callers invoke `civ_build::pick_tile_set` in
`ArchitectureMode::Primitive` and the supplied `primitive_tile_set_id` does
not match any `TileSetProfile` in the input slice, the resolver must fall
back deterministically rather than panicking or silently returning `None`.
The fallback must be replay-stable: identical inputs (same vector, same
demand signals, same tile-set slice, same ordering) produce the same
selected id across runs and across builds.

### What This FR Achieves

The fallback path keeps the rest of `pick_tile_set`'s contract intact
(`Option<u16>` return, culture/era/wealth filtering, parcel-template
score) while substituting a sensible stable selection for the missing
primitive id. Verified by `fr_civ_core_021_missing_primitive_id_falls_back_
deterministically` in `crates/build/tests/fr_matrix_batch12.rs`, which
runs the resolver against two input orderings of the same tile-set slice
and asserts equality.

## Acceptance Signal

- `cargo test -p build fr_civ_core_021` passes.
- Reversed input ordering yields the same selected id.
- Missing `primitive_tile_set_id` falls through to the canonical tile-set
  selection path of the same vector.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/build/` |
| Resolver | `crates/build/src/lib.rs:pick_tile_set` |
| Test | `crates/build/tests/fr_matrix_batch12.rs:765` |
