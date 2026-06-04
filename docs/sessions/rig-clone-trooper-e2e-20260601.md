# Rig Clone-Trooper End-to-End (#991, scoped to ONE unit)

**Date:** 2026-06-01
**Branch:** `feat/rig-clone-trooper-20260601` (off `reconcile3/converge-20260531`)
**Goal:** Make ONE SW unit (clone-trooper, `vanilla_mapping: line_infantry`) pass the skinned-mesh
swap guard and render/animate live.

## Result (plain answer)

**Did one unit finally swap visibly? NO — but the documented "last blocker" (the bindpose guard)
is solved, and a DIFFERENT, deeper blocker was isolated: DINO's `AssetBundle.LoadFromFile`
rejects the freshly-built `sw-clone-trooper-republic` bundle (returns null) regardless of build
permutation, so the swap never reaches the (now-satisfiable) guard.**

The live log line is:
```
[AssetSwap] TrySwapRenderMeshFromBundle: bundle 'sw-clone-trooper-republic' failed to load and
            could not be recovered — marking permanently failed (#992).
[AssetSwap] AssetSwapSystem: swap failed — address='sw-clone-trooper-republic' (attempt 1/200)
```
No `swapped N/100`; no `static (0 bindposes)` skip — it never loads.

## What WAS achieved (each pipeline stage)

### 1. DINO infantry skeleton spec — DONE
- Built `src/Tools/SkeletonExtractor/` (AssetsTools.NET 3.0.4 + UABEA `classdata.tpk`).
- DINO unit meshes are in **`settings_assets_all.bundle`** (1178 meshes), NOT
  `defaultlocalgroup` (MonoBehaviours/AnimationClips only).
- **`dark_knight` = exactly 21 bindposes / 21 bone-name-hashes**, confirming the guard log.
- Full ordered hash fingerprint + method in `docs/sessions/dino-infantry-skeleton-20260601.md`.

### 2. Rig the clone mesh to 21 bones — DONE
- `packs/warfare-starwars/assets/tools/blender_rig_to_dino_skeleton.py` (Blender 4.5):
  - Raw clone GLB was found to be **already skinned (66 joints)** — the R&D doc's "SW meshes are
    static" premise is wrong for this asset; the *bundle* held an `Icosphere` placeholder.
  - Builds a fresh 21-bone humanoid armature, binds the mesh (auto-weights; heat-weighting fully
    failed so a nearest-bone fallback assigns all 6170 verts), decimates to ~2000 tris.
  - **Output GLB: skin with 21 joints + inverseBindMatrices.** Key fixes: decimate before bind,
    `export_apply=False`, mark `use_deform`, and the nearest-bone weight fallback (heat weighting
    returned 0% coverage).
- Verified the resulting **Unity mesh has `bindposes.Length == 21`** (logged at bundle build).

### 3. f2 bundle built + deployed — DONE (build), but bundle won't load (see below)
- `unity-assetbundle-builder/Assets/Editor/BuildRiggedCloneTrooper.cs` imports the rigged GLB
  (via glTFast `GltfImporter`), URP/Lit material, prefab keyed `sw-clone-trooper-republic`,
  builds for StandaloneWindows64 with Unity **2021.3.45f2**. Bundle deployed to pack + game.
- The SkeletonExtractor confirms the deployed bundle contains **3 skinned meshes, each 21
  bindposes** — i.e. it WOULD satisfy `IsSkinnedMeshCompatible`.

### 4 + 5. Live verify — bundle fails to load (NEW blocker)

Ran 5 controlled rebuild/redeploy/restart cycles, each a clean DINO restart + Sandbox +
reload-packs, reading the swap log:

| Build variant | UnityFS ver | Class profile | Live `LoadFromFile` |
|---|---|---|---|
| ChunkCompressed, SkinnedMeshRenderer | f2 | 43,137,21,48 | **null (fail)** |
| Uncompressed, SkinnedMeshRenderer | f2 | 43,137,… | **null (fail)** |
| Native-rebaked mesh, SkinnedMeshRenderer | f2 | 43,137,… | **null (fail)** |
| Static-carrier (MeshFilter+MeshRenderer), 21-bindpose mesh | f2 | 1,4,21,23,33,43,48 | **null (fail)** |
| Static-carrier, **f1** | f1 | 1,4,21,23,33,43,48 | **null (fail)** |
| Static-carrier, manifest-driven build overload | f2 | 1,4,21,23,33,43,48 | **null (fail)** |

Six permutations, identical failure. The blocker is in this builder project's bundle OUTPUT, not
the build API, compression, Unity version, or class profile.

Control facts from the same live sessions:
- The OLD primitive `sw-clone-trooper-republic` (Icosphere) bundle **loaded** historically.
- `sw-clone-heavy` (**f1**, static, class profile `1,4,21,23,33,43,48`) **loads** right now.
- `sw-rep-clone-trooper` (**f2**, static) **loads** (extracts its Icosphere).

So neither the Unity version (f1 AND f2 both load for OTHER bundles), nor the skinned-vs-static
class profile (my static-carrier matches the loadable `sw-clone-heavy` exactly), nor compression,
explains it. The remaining un-eliminated variable is **something this specific builder project
serializes into the bundle** (the project is f1-origin, upgraded to f2, with `com.unity.cloud.gltfast`
6.0.1 + URP 12.1.12 installed). Every bundle this project emits fails `LoadFromFile`; the
pre-existing loadable bundles were produced by a different toolchain.

This means the task's stated "last blocker" (bindpose guard) is NOT actually the last blocker for
a freshly-built bundle — a bundle-load incompatibility sits in front of it.

## Runtime hardening landed (real fix, retained)

`src/Runtime/Bridge/AssetSwapSystem.cs` `LoadBundle`: added Unload-lag recovery for the
**null-return** path (Unity returns null — not the documented "already loaded" throw — when the
LRU evicted+Unloaded a bundle whose file Unity still holds; with ~50 SW bundles and a 10-slot LRU
this silently failed ~48/50 swaps). Now reuses the still-loaded handle via `FindLoadedBundleByPath`.
This did NOT recover the clone bundle (Unity has no handle for it = genuine load failure), but it
is a correct fix for the broader per-frame flood and is kept.

## Next step to actually land the swap

The bundle-load failure is now the single gating issue. Recommended: build the bundle with the
**toolchain that produced the loadable bundles** (find/replicate whatever made `sw-clone-heavy` /
the old primitive bundles — a clean f2 project WITHOUT the glTFast/URP package soup, importing the
rigged GLB via a minimal path), or determine via a standalone `AssetBundle.LoadFromFile` harness
exactly why this project's bundles are rejected. Once any bundle from the rig pipeline loads, the
21-bindpose mesh will pass `IsSkinnedMeshCompatible` and the swap should report `swapped N/100`.

## Files

- `src/Tools/SkeletonExtractor/` — DINO bundle skeleton extractor (new tool)
- `docs/sessions/dino-infantry-skeleton-20260601.md` — skeleton spec (21 bones)
- `packs/warfare-starwars/assets/tools/blender_rig_to_dino_skeleton.py` — 21-bone rig script
- `unity-assetbundle-builder/Assets/Editor/BuildRiggedCloneTrooper.cs` — f2 bundle builder
- `packs/warfare-starwars/assets/working/sw_clone_trooper_phase2_sketchfab_001/rigged_21bone.glb`
- `packs/warfare-starwars/assets/bundles/sw-clone-trooper-republic` — rebuilt skinned bundle
- `src/Runtime/Bridge/AssetSwapSystem.cs` — LoadBundle null-return Unload-lag recovery
