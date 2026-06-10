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
| `road.glb`      | road tile           | `GltfAssetLabel::Scene(0)` |
| `rock.glb`      | rock decoration     | `GltfAssetLabel::Scene(0)` |
| `cart.glb`      | vehicle             | `GltfAssetLabel::Scene(0)` |

Paths are defined as constants in `src/gltf_models.rs::asset_paths` so the
fetch script and the loader stay in sync. Bevy resolves them relative to this
crate's `assets/` root.

## Expanded variety (new — wire into `gltf_models.rs` when ready)

The fetch script now also lands extra CC0 variety for emergent worlds. These
are **not yet referenced** by `src/gltf_models.rs` (owned elsewhere); add new
`asset_paths` constants + `GameModels` slots to wire them in. All load as
`GltfAssetLabel::Scene(0)`.

| File                            | Kind        | Source model                  |
| ------------------------------- | ----------- | ----------------------------- |
| `building_house_B.glb`          | building    | KayKit `building_home_B`      |
| `building_tower.glb`            | building    | KayKit `building_tower_A`     |
| `building_church.glb`           | building    | KayKit `building_church` (temple) |
| `building_market.glb`           | building    | KayKit `building_market`      |
| `building_tavern.glb`           | building    | KayKit `building_tavern` (hut/inn) |
| `building_well.glb`             | building    | KayKit `building_well`        |
| `tree_b.glb`                    | nature      | KayKit `tree_single_B`        |
| `tree_large.glb`                | nature      | KayKit `trees_A_large`        |
| `rock_b.glb`                    | nature      | KayKit `rock_single_B`        |
| `rock_c.glb`                    | nature      | KayKit `rock_single_C`        |
| `creature_skeleton_minion.glb`  | creature    | KayKit `Skeleton_Minion`      |
| `creature_skeleton_warrior.glb` | creature    | KayKit `Skeleton_Warrior`     |
| `cart_wheelbarrow.glb`          | vehicle     | KayKit `wheelbarrow`          |
| `boat.glb`                      | vehicle     | Quaternius Pirate Kit (manual drop — see LICENSE.txt) |

Creatures provide CC0 non-human life forms for emergent fauna; buildings give
hut/house/tower/temple/market variety for emergent settlements.

## Licensing

Only CC0 / public-domain assets. Suggested sources:

- Quaternius — https://quaternius.com (CC0)
- Kenney — https://kenney.nl/assets (CC0)

Keep attribution/provenance notes here when the fetch script lands the files.
