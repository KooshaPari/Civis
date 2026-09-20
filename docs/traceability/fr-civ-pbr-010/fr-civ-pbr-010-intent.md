# Intent: FR-CIV-PBR-010 -- WGSL triplanar shader integration

> Date: 2026-09-19
> FR: FR-CIV-PBR-010
> Epic: FR-CIV-PBR

## User Intent

The Phase-3 PBR work needs to land a WGSL triplanar shader at
`assets/shaders/pbr_triplanar.wgsl` and surface it through the Bevy
adapter. The shader samples albedo / normal / metallic-roughness from
three orthogonal world-axis projections, blends them with the squared-
and-normalised world normal, and optionally adds a second triplanar set
at lower weight for detail texturing without UV stretching. The Bevy
adapter loads the shader source via `include_str!` so the shader and
the Rust substrate always agree on byte-for-byte contents.

### What This FR Achieves

`crates/voxel/src/material_pbr.rs` documents the integration and exposes
`MaterialCatalog::material_count()` covering the **union** of distinct
`MaterialId`s across the triplanar splat plan and the greedy atlas plan
(overlapping matids counted once). The Bevy adapter uses
`material_count()` to size the bind-group scratch and the atlas backing
storage before the first draw.

## Acceptance Signal

- `cargo test -p voxel fr_pbr_phase3_material_catalog_counts_union_of_matids`
  passes.
- Empty catalog returns `material_count() == 0`.
- `assets/shaders/pbr_triplanar.wgsl` ships and is referenced via
  `include_str!`.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/voxel/` |
| Catalog aggregator | `crates/voxel/src/material_pbr.rs:14` |
| Tests | `crates/voxel/src/material_pbr.rs:1560`, `:1594` |
| Shader | `assets/shaders/pbr_triplanar.wgsl` |
