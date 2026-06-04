# Native Ocean Plan for Civis

## Goal
Replace the current flat `Plane3d` water placeholder with a native Bevy 0.18 + wgpu (DX12) ocean stack. The implementation strategy is `wrap > handroll`: start by wrapping `bevy_water` 0.18.1, then layer in custom wgpu/WGSL systems where the upstream crate stops.

## Constraints
- Civis uses Bevy 0.18 and wgpu on DX12.
- `bevy_water` 0.18.1 is confirmed compatible with Bevy 0.18.
- The ocean must sit at Civis sea level: `WORLD_DIMS[1] * 0.40`.
- Water depth color must sample voxel terrain `surface_y`, not a flat global depth heuristic.
- Later polish should project caustics onto the voxel seabed mesh.

## Phased WBS

| Phase | Task | Depends On | Outcome |
|---|---|---:|---|
| P1 | Wrap `bevy_water` and replace the placeholder plane | - | Native water base with Gerstner waves, Fresnel, foam, and depth color |
| P2 | Add higher-end ocean rendering layers in custom wgpu/WGSL | P1 | Open-ocean realism via spectral simulation and reflections |
| P3 | Add polish and world interaction layers | P1, P2 | Shipping-quality water with caustics, foam breakup, LOD, and fog |

## DAG
- `P1` starts the dependency chain.
- `P2` depends on `P1` because the wrapped water plugin should own the baseline surface, uniforms, and depth contract before compute/reflection layers are added.
- `P3` depends on `P1` and `P2` because polish features consume the same sea-level, terrain-depth, and reflection inputs established earlier.

## P1: Wrap `bevy_water`

### Target
Swap the flat plane for `bevy_water` and adapt it to Civis world data.

### Techniques

| Technique | Difficulty | Forkable Rust/wgpu crate exists | Implementation sketch |
|---|---|---|---|
| Gerstner waves | Small | Yes, `bevy_water` 0.18.1 | Wrap the crate as the base ocean plugin, keep its animated surface pipeline, and map wave controls to Civis water settings instead of a standalone plane mesh. |
| Fresnel | Small | Yes, inside `bevy_water` or as a light wrapper | Use the fragment view-angle term to blend reflection and refraction, keeping the material physically plausible without introducing a custom reflection pass yet. |
| Foam | Small | Yes, inside `bevy_water` or as a light wrapper | Derive foam from wave steepness and shallow-water cues, then expose a mask that can be tuned from Civis gameplay or weather state. |
| Depth-color base | Small | Yes, inside `bevy_water` or as a light wrapper | Read the voxel `surface_y` for each water column and tint water by depth relative to that terrain height, replacing the current flat-color assumption. |

### Civis integration points
- Place the water surface at `WORLD_DIMS[1] * 0.40` so the rendered ocean aligns with Civis sea level.
- Read voxel `surface_y` for depth-color evaluation so shoreline, shelves, and bays respond to the actual terrain.
- Replace the existing `Plane3d` spawn path with a wrapped water plugin and a water-surface entity/material setup.

### Cargo snippet for P1

```toml
# P1: wrap the Bevy 0.18-compatible base water crate first.
# bevy_water = "0.18.1"
```

## P2: SOTA layers via custom wgpu/WGSL

### Target
Move beyond the base wrapper with open-ocean simulation and better reflections.

| Technique | Difficulty | Forkable Rust/wgpu crate exists | Implementation sketch |
|---|---|---|---|
| FFT/Tessendorf spectrum | Medium-hard | Yes, via Rust/wgpu FFT building blocks and shader examples | Run a compute pass that evolves the ocean spectrum in frequency space, then inverse-transform it into displacement, normals, and roughness inputs for the surface material. |
| Screen-space reflections | Medium-hard | Yes, but usually as custom render-graph work rather than a drop-in crate | Use the depth buffer and view vectors to reconstruct reflection samples in screen space, then fall back to planar reflection when the angle or coverage breaks SSR quality. |
| Planar reflections | Medium-hard | Yes, but usually custom in Bevy/wgpu | Render a mirrored camera pass into an offscreen texture for the ocean plane, then blend it with Fresnel so the water can hold a stable horizon reflection. |

### Civis integration points
- Keep the sea-level uniform from P1 as the reflection plane origin.
- Continue using voxel `surface_y` as the authoritative depth source for under-water tint, foam thresholds, and shoreline attenuation.
- Expose a reflection target that the terrain and sky systems can sample without coupling the ocean pass to the rest of the renderer.

## P3: Polish

### Target
Add the final layers that make the ocean feel grounded in Civis terrain and lighting.

| Technique | Difficulty | Forkable Rust/wgpu crate exists | Implementation sketch |
|---|---|---|---|
| Animated caustics on seabed | Medium | Partially, often custom shader work | Project an animated caustic pattern onto the voxel seabed mesh using world-space coordinates and depth under water, then modulate color and intensity by terrain slope and light direction. |
| Jacobian breaking foam | Medium | Usually hand-rolled | Compute local wave compression from the displacement field Jacobian, then spawn foam where the surface is stretching or breaking so crests look energetic instead of uniformly noisy. |
| LOD clipmap rings | Medium | Yes, Rust/wgpu clipmap approaches exist | Use concentric clipmap rings or similar far-field tessellation so near-camera water stays dense while the horizon remains cheap and stable. |
| Horizon atmospheric fog | Medium | Partially, usually custom | Blend water into atmospheric fog by distance and view height so the ocean horizon dissolves naturally into Civis sky lighting. |

### Civis integration points
- Project caustics onto the voxel seabed mesh, not just the water surface, so sunlight reads as terrain interaction.
- Use `surface_y` again for the underwater projection and attenuation logic, since that is the stable terrain-facing depth contract.
- Keep the far-field LOD and fog tuned to Civis camera scale so the ocean horizon does not expose clip edges or repetitive tiling.

## Technique matrix

| Technique | Phase | Difficulty | Forkable Rust/wgpu crate exists | Notes |
|---|---|---|---|---|
| Gerstner waves | P1 | Small | Yes | Best handled by `bevy_water` as the first wrapped base. |
| Fresnel | P1 | Small | Yes | Needed immediately for believable reflectance. |
| Foam | P1 | Small | Yes | Good first-pass shoreline and crest breakup. |
| Depth-color base | P1 | Small | Yes | Must key off voxel `surface_y`. |
| FFT/Tessendorf spectrum | P2 | Medium-hard | Yes | Highest-value realism upgrade for open water. |
| Screen-space reflections | P2 | Medium-hard | Yes | Good quality-per-cost when the camera sees the surface directly. |
| Planar reflections | P2 | Medium-hard | Yes | Better stability for the horizon and steep viewing angles. |
| Animated caustics | P3 | Medium | Partially | Strong visual payoff on voxel seabeds. |
| Jacobian foam | P3 | Medium | Usually hand-rolled | Makes breaking waves read correctly. |
| LOD clipmap rings | P3 | Medium | Yes | Keeps large oceans affordable. |
| Horizon fog | P3 | Medium | Partially | Hides far-field repetition and hard horizon seams. |

## Recommended execution order
1. P1 first: wrap `bevy_water`, replace the `Plane3d`, and lock sea-level plus depth-color integration to Civis terrain data.
2. P2 next: add FFT/Tessendorf and reflection infrastructure once the baseline ocean is stable.
3. P3 last: layer in caustics, breaking foam, clipmap LOD, and horizon fog after the base shading model is proven.

## Expected outcome
- A native Civis ocean that starts with a small integration surface instead of a from-scratch water engine.
- A clear upgrade path from base water realism to higher-end ocean simulation.
- A stable integration contract with sea level, voxel depth sampling, and seabed projection that other systems can reuse.
