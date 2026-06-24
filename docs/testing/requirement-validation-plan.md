# Requirement-validation plan: BDD layer for `civ-bevy-ref`

This module adds acceptance checks at requirement level (what the shipped game must do), not unit-level component behavior.

## Why BDD here

Current unit tests in `crates/voxel` and `clients/bevy-ref` verify local behavior, but they can pass while user-facing requirements are broken.

The `requirements_bdd` suite defines explicit Given–When–Then scenarios over the public game wiring APIs and worldgen contract:

1. World-size choice must map to monotonic dimensions.
2. Starting a new world must produce a genuinely different world for a different seed.
3. 2D map raster extent must match 3D world extent.
4. Water must not appear above sea level.
5. Terrain meshing should be continuous, not blob-like.

## Test mapping

- `requirement_world_size_selection_changes_dimensions`
  - Given: size indices `0..N`.
  - When: `world_dims_for(index)` is invoked.
  - Then: `[x,y,z]` dimensions must strictly increase with index.
  - Gap closed: catches a hardcoded dimension bug where size selection UI does not actually drive generation size.
  - Status: `#[ignore]` until world-size mapping API is public (currently private).

- `requirement_new_world_differs_from_previous`
  - Given: two different world seeds.
  - When: `worldgen::generate(dims, seed)` runs for each.
  - Then: surface signatures differ on > 20% of sampled columns.
  - Gap closed: detects “new world” producing identical terrain regardless of input seed.

- `requirement_2d_map_extent_matches_world`
  - Given: a generated world with size `D`.
  - When: basemap image/coverage is produced.
  - Then: the map must sample full `[0..D.x] × [0..D.z]` extents.
  - Gap closed: catches 2D rasterization extent clipping that would produce partial/smaller previews.
  - Status: `#[ignore]` until basemap extent or renderer entrypoint is public.

- `requirement_water_only_below_sea`
  - Given: any generated world and computed sea level.
  - When: all water cells are enumerated.
  - Then: every WATER voxel must satisfy `y <= sea_level`.
  - Gap closed: enforces the visible-water contract.

- `requirement_terrain_is_continuous_not_blobs`
  - Given: a generated chunk sampled from dense terrain region.
  - When: that chunk is meshed with `CubicMesher`.
  - Then: mesh `vertices`/`indices` counts must exceed continuity thresholds.
  - Gap closed: rejects tiny/fragmented mesh outputs from blob-like terrain fill.

## Notes

- The ignored tests document where missing pub API blocks end-to-end wiring checks:
  - `clients/bevy-ref/src/voxel_sim.rs::world_dims_for` is private.
  - `clients/bevy-ref/src/map2d.rs` basemap image/sampling helpers are private.
- Once those are made public, flip the ignored scenarios to active by removing `#[ignore]` and replacing placeholders with concrete assertions.
