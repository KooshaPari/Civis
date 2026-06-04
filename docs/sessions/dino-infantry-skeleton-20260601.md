# DINO Infantry Skeleton Spec (#991)

**Date:** 2026-06-01
**Branch:** `feat/rig-clone-trooper-20260601` (off `reconcile3/converge-20260531`)
**Goal:** Reverse-engineer the DINO vanilla infantry skeleton that `rep_clone_trooper`
(`vanilla_mapping: line_infantry`) replaces, so the Star Wars clone-trooper mesh can be
rigged to a matching bindpose count and pass `AssetSwapSystem.IsSkinnedMeshCompatible`
(`AssetSwapSystem.cs` L1059).

---

## 1. How it was extracted

DINO ships its unit meshes inside Unity Addressables bundles under
`Diplomacy is Not an Option_Data/StreamingAssets/aa/StandaloneWindows64/`. The meshes are
**not** in `defaultlocalgroup_assets_all.bundle` (that holds MonoBehaviours / AnimationClips,
typeIds 114/74 — 0 meshes). They live in **`settings_assets_all.bundle`** (1.5 GB, 1178
skinned/static meshes).

The bundles strip type trees, so deserialization needs a class package. Tool built for this:

- `src/Tools/SkeletonExtractor/` — AssetsTools.NET 3.0.4 reader. Loads `classdata.tpk`
  (Unity 2021.3.45f2 class DB, sourced from UABEA v8), enumerates `AssetClassID.Mesh`,
  reads `m_BindPose` (array size) + `m_BoneNameHashes`. `SMR=1` env var additionally walks
  `SkinnedMeshRenderer.m_Bones → Transform → GameObject.m_Name` for human-readable bone names.

Run:
```
SMR=1 skeleton-extractor "<settings_assets_all.bundle>" dark_knight skeleton_dark_knight.json
```

`classdata.tpk` must sit next to the exe (`bin/Release/net11.0/`). Not committed (289 KB binary,
re-downloadable from UABEA release v8 `uabea-windows.zip`).

---

## 2. The skeleton — `dark_knight` (the guard's named vanilla infantry mesh)

The runtime guard log named `dark_knight` with **21 bindposes** as the vanilla mesh that the
clone-trooper bundle was being rejected against. Extraction confirms it exactly.

| Property | Value |
|---|---|
| Mesh name | `dark_knight` |
| **Bindpose count** | **21** |
| Bone-name-hash count | 21 |
| SubMesh count | 1 |
| Source bundle | `settings_assets_all.bundle` |
| Unity version | 2021.3.45f2 |

### Ordered bone-name hashes (m_BoneNameHashes, CRC of bone transform path)

Index order matches `m_BindPose` order — this is the authoritative bone ordering DINO's
procedural animation systems index into:

```
[ 3297942148, 3750120588, 1385759235, 2615517091, 4186151569,
  113222829,  2910317188, 1811059477, 254416586,  59549682,
  1436622580, 4112853929, 4013327453, 603097146,  1647331369,
  3657282905, 2098613160, 2589172191, 2554377818, 3127574138,
  14941120 ]
```

Human-readable bone names were not recoverable from `settings_assets_all.bundle` alone — the
SkinnedMeshRenderer + Transform/GameObject hierarchy that owns the `dark_knight` mesh lives in
the unit *prefab* (cross-bundle reference into `defaultlocalgroup`), and no in-bundle SMR
references the mesh PathID directly. The hashes above are nonetheless a complete, ordered
fingerprint: a 21-bone humanoid infantry rig. (The hashes are not plain `CRC32(name)` of common
bone names — Unity hashes the full transform path with its internal algorithm.)

### Cross-check: bindpose counts are per body-class, not per-cosmetic

The runtime guard log shows other vanilla meshes with their own counts, confirming a small set
of shared rigs by body class (the R&D doc's "shared rig" hypothesis):

| Vanilla mesh | Bindposes | Class |
|---|---|---|
| `dark_knight` | 21 | humanoid infantry ← **clone-trooper target** |
| `ballista` | 55 | siege/large |
| `weath_floor.009` | 3 | structure/prop |

**Implication:** to swap a SW mesh onto a `line_infantry` (humanoid) entity, the SW mesh must be
a skinned mesh with exactly **21 bindposes**.

---

## 3. What this unblocks

`IsSkinnedMeshCompatible(current, replacement)` requires:
1. `replacement.bindposes.Length > 0` (not static), AND
2. `replacement.bindposes.Length == current.bindposes.Length` (== 21 for infantry).

The clone-trooper replacement therefore must be rigged to **21 bindposes** and bundled as a
skinned mesh. See the rigging step (`blender_rig_to_dino_skeleton.py`) and the rebuilt bundle
`sw-clone-trooper-republic`.
