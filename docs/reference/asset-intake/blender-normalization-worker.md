# Blender Normalization Worker Spec

## Purpose

Define the standard working conversion step between raw sourced assets and Unity-ready artifacts.

## Inputs

- `raw/<asset_id>/source_download.*`
- Source metadata snapshot (`asset_manifest.json`)
- Optional preview images for manual QA

## Outputs

- `working/<asset_id>/normalized.blend`
- `working/<asset_id>/normalized.glb`
- `working/<asset_id>/validation_report.json`
- `working/<asset_id>/preview.png`

## Standard Steps

1. Import raw asset into Blender.
2. Apply/normalize transforms (scale, forward/up axes, center pivot policy).
3. Rename objects and materials into deterministic schema-compatible names.
4. Merge duplicate materials and flatten texture naming.
5. Optional rig conversion for animation-ready assets.
6. LOD/decimate pass toward target budget:
   - Infantry: 1k–6k tris
   - Elite infantry/hero: 4k–12k tris
   - Small vehicle: 3k–15k tris
   - Large vehicle: 10k–40k tris
7. Export GLB + Blender source; emit preview.

## Validation Hooks

- Missing textures check
- Non-manifold geometry
- Degenerate scale check
- Bone/rig consistency if `rigged=true`
- Texture resolution budget warning

Hard fail on:

- corrupt import
- zero-geometry output
- missing provenance metadata

## Policy Controls

- All jobs read profile from `--profile`.
- Profiles map to stylistic transforms:
  - `dinosw_lowpoly_v1`: aggressive edge simplification, material flattening, hard silhouette preservation.
  - `prototype_fast_v1`: faster import path with lower cleanup strictness.
