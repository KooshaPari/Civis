# Brickalyzer / Legolizer R&D

Date: 2026-05-31

## Feasibility verdict

Feasible as an alternate art mode.

The lowest-risk path is to treat "brick" as a procedural replacement layer that sits on top of the existing bundle-to-AssetSwap pipeline:

1. Read the source mesh from the bundle or import target.
2. Voxelize it against the mesh bounding box using a fixed brick grid.
3. Emit one URP/Lit cube per filled voxel.
4. Merge the cubes with `CombineInstance` into a single renderable mesh.
5. Assign a faction-colored URP/Lit material so the result stays Hybrid Renderer V2-safe.

That keeps the mode aligned with the current asset pipeline rather than introducing a parallel content system.

## Why this avoids the Standard-shader crash

The crash path we already hit came from legacy `Standard` materials being written into a URP / HRV2 render path.

This brick mode avoids that in two ways:

- The generated material is `Universal Render Pipeline/Lit`, not `Standard`.
- The pipeline only produces URP-compatible mesh/material pairs, so AssetSwap can hand them to the live render path without the legacy shader mismatch.

In practice, the brick mode should be safer than swapping arbitrary authored meshes because the output is procedural and shader-normalized at generation time.

## Integration point

This can plug into the existing `bundle -> AssetSwap` flow as a pre-export step:

- bundle import still identifies the source asset
- the Brickalyzer prototype converts the source mesh into a brick mesh
- the resulting mesh/material can be written back through the same bundle swap and live entity swap mechanisms already used by AssetSwap

No new runtime content registry is required for the first pass.

## GraphicsMode recommendation

Recommend making brick a third tier in `GraphicsMode`:

- `low-poly` = current vanilla look
- `realistic` = URP post-process + material upgrade path
- `brick` = voxelized alt look with faction-colored URP/Lit cubes

That gives the user a clear quality/style axis instead of making brick a separate feature flag.

## Effort estimate

- Prototype editor script: 0.5 to 1 day
- Bundle integration and AssetSwap wiring: 1 to 2 days
- Faction tint plumbing and cleanup: 0.5 day
- Performance tuning and artifact cleanup: 1 to 2 days

Total for a solid first-pass mode: about 3 to 5 engineer-days.

## Risks

- Dense meshes can explode voxel counts if the grid is too fine.
- Small details will alias away unless the voxel size is tuned per asset class.
- Merge cost can climb on large buildings unless the prototype uses a conservative fill threshold.

## Prototype status

The editor prototype is intentionally narrow:

- it is editor-only
- it converts the selected mesh into a brick-voxelized URP mesh
- it uses the same URP/Lit material family as the crash-safe runtime path
- it is suitable as the first step before bundle export integration
