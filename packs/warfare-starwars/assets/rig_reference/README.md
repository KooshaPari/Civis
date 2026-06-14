# DINO reference rigs (RIGGING-OPTIMIZER #991)

Per-archetype reference meshes and bone name lists for retargeting Star Wars unit GLBs onto
DINO-compatible skeletons. Populate during **P0 — Observe** (vanilla mesh diagnostic survey).

## Profiles

| Profile ID | Vanilla unit class | Reference source | Status |
|------------|-------------------|------------------|--------|
| `dino_humanoid_infantry` | line_infantry, ranged_infantry, melee | TBD from diagnostic `mesh="..."` survey | pending |
| `dino_droid_humanoid` | CIS droid infantry | TBD | pending |
| `dino_walker` | walkers / large vehicles | static or articulated TBD | pending |
| `static` | flyers, props | no bindposes | n/a |

## Per-profile artifacts

```
rig_reference/<profile_id>/
  reference_mesh.glb      # exported from DINO Addressables / diagnostic capture
  bone_names.json         # ordered bone names matching Mesh.bindposes
  reference_rigged.blend  # Blender source of truth for weight transfer
```

## Blender batch

```powershell
blender --background --python packs/warfare-starwars/assets/tools/blender_rig_and_decimate.py -- `
  --source packs/warfare-starwars/assets/raw/<asset_id>/model.glb `
  --reference packs/warfare-starwars/assets/rig_reference/dino_humanoid_infantry/reference_mesh.glb `
  --output packs/warfare-starwars/assets/working/<asset_id>/rigged.glb `
  --lod0-tris 2000
```

See `docs/sessions/rigging-optimizer-pipeline-20260531.md` for full pipeline design.
