# CC0 GLTF models

These `.glb` files back the `models` cargo feature (`src/gltf_models.rs`,
`GltfModelsPlugin` + `GameModels`). When present they replace the procedural
primitives spawned by `sim_bridge` (capsule civilians, cuboid buildings,
cone trees). When absent, the loader stores `None` and callers fall back to
the primitives — the build and runtime stay healthy either way.

## Expected files

The asset-pipeline agent's `fetch-cc0-models` script populates these from CC0
sources (Quaternius / Kenney — public-domain / CC0):

| File            | Replaces            | Loaded as              |
| --------------- | ------------------- | ---------------------- |
| `civilian.glb`  | capsule civilian    | `GltfAssetLabel::Scene(0)` |
| `tree.glb`      | cone tree decoration| `GltfAssetLabel::Scene(0)` |
| `building.glb`  | cuboid building     | `GltfAssetLabel::Scene(0)` |

Paths are defined as constants in `src/gltf_models.rs::asset_paths` so the
fetch script and the loader stay in sync. Bevy resolves them relative to this
crate's `assets/` root.

## Licensing

Only CC0 / public-domain assets. Suggested sources:

- Quaternius — https://quaternius.com (CC0)
- Kenney — https://kenney.nl/assets (CC0)

Keep attribution/provenance notes here when the fetch script lands the files.
