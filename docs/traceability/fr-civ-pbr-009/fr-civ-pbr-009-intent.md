# Intent: FR-CIV-PBR-009 -- Pure triplanar PBR blend helper

> Date: 2026-09-19
> FR: FR-CIV-PBR-009
> Epic: FR-CIV-PBR

## User Intent

The Phase-3 PBR substrate in `crates/voxel/src/material_pbr.rs` needs a
fully deterministic, engine-agnostic **blend helper** that consumes three
pre-fetched axis samples (per-axis albedo-linear / perceptual roughness /
unit-length normal) and emits the blended result, ready to feed into a
shading pipeline without any I/O, RNG, or hidden state.

### What This FR Achieves

Three types and two functions, all under `crates/voxel/src/material_pbr.rs:754+`:

- `TriplanarAxisSample { albedo_linear, perceptual_roughness, normal }`
- `TriplanarPbrBlend` (blended output)
- `triplanar_axis_weights(world_normal)` — absolute normal components
  normalised to sum to `1.0`; degenerates (zero normal) to equal axis
  weights so the helper stays total and replay-safe.
- `blend_triplanar_pbr(weights, x, y, z)` — linear blend of albedo /
  roughness by axis weights; linear blend of normals followed by
  renormalisation.

## Acceptance Signal

- `cargo test -p voxel fr_pbr_009_triplanar_pbr_blend_is_deterministic_and_normalized`
  passes (asserts exact weights, blended channels, normal renormalisation).
- `triplanar_axis_weights([0,0,0]) == [1/3, 1/3, 1/3]`.
- No `bevy`, `wgpu`, or RNG dependencies are introduced.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/voxel/` |
| Helper | `crates/voxel/src/material_pbr.rs:754` |
| Test | `crates/voxel/src/material_pbr.rs:1461` |
