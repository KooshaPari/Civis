# Unity Import Contract

## Purpose

Standardize the Unity side artifacts from the intake pipeline so runtime pack tooling can consume a consistent layout.

## Canonical Export Layout

```text
export/unity/
  units/<asset_id>/<asset_id>.fbx
  units/<asset_id>/<asset_id>.json
  units/<asset_id>/preview.png
  vehicles/<asset_id>/<asset_id>.fbx
  vehicles/<asset_id>/<asset_id>.json
  props/<asset_id>/<asset_id>.fbx
  props/<asset_id>/<asset_id>.json
```

## Data Contract (`<asset_id>.json`)

```json
{
  "asset_id": "sw_tie_fighter_sketchfab_001",
  "fidelity_profile": "dinosw_lowpoly_v1",
  "animation": {
    "has_animations": false,
    "anim_names": []
  },
  "render": {
    "triangle_count": 4200,
    "material_count": 4,
    "pivot": [0.0, 0.0, 0.0]
  },
  "source": {
    "platform": "sketchfab",
    "original_format": "glb",
    "download_url": "..."
  },
  "manifest_ref": "asset_manifest.json"
}
```

## Import Expectations

- FBX exported with axis mapping matching game conventions.
- Units may carry reduced LOD variants under same manifest.
- All textures embedded or colocated with deterministic naming.
- Materials remain readable and palette-consistent; no dependency on source shader graphs.

## Pack Integration

Each export unit maps to gameplay tags via manifest `tags` and `category`, and is consumed by later registry steps that already exist in the pack pipeline.
