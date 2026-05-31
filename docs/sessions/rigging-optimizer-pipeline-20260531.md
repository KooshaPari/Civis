# RIGGING-OPTIMIZER Pipeline (Task #991)

**Date:** 2026-05-31  
**Branch:** `feat/rigging-optimizer-20260531` (worktree `.claude/worktrees/rigopt`)  
**Related:** #973 (procedural unit animation + spark VFX), #991 (rigging-optimizer)  
**Problem:** Swapped Star Wars unit meshes appear **static/frozen** while vanilla DINO units animate.

---

## Executive summary

| Layer | Finding |
|-------|---------|
| **DINO rendering** | Gameplay units use ECS **`RenderMesh`** shared components (Hybrid Renderer). Animation is driven by DINO's internal systems (`MmAnimation*`, procedural bone updates), **not** by swapping `GameObject` `SkinnedMeshRenderer` instances in the scene. |
| **SW assets today** | Sketchfab GLBs are imported as **static** meshes (`Rigging: None` per `UNITY_IMPORT_GUIDE.md`). Bundles expose `MeshFilter`/`MeshRenderer` or an SMR whose `sharedMesh` has **no bindposes / bone weights**. |
| **AssetSwapSystem** | Phase 2 copies only `Mesh` + `Material` onto `RenderMesh` via reflection. It **does not** rebind bones or validate skinning compatibility. Assigning a static mesh to an entity whose vanilla mesh was skinned produces a **frozen** silhouette (bind pose + procedural motion mismatch — #973). |
| **Fix order** | (1) Rig/retarget SW meshes to DINO's reference skeleton per archetype → (2) decimate + LOD → (3) bundle as skinned meshes → (4) extend swap path with bindpose validation (prototype landed) → (5) populate `BundleToVanillaMeshMap` from diagnostic survey. |

---

## 1. DINO unit skinning model (findings)

### 1.1 What we know from code and docs

- **`AssetSwapSystem`** documents entity-dump conclusion: DINO uses `Unity.Rendering.RenderMesh` on unit archetypes.
- **Live swap path** mutates `RenderMesh.mesh` and `RenderMesh.material` only (`TrySwapRenderMeshFromBundle`). No `SkinnedMeshRenderer` on ECS entities; no `Animator` in the swap loop.
- **Prefab extraction** prefers `SkinnedMeshRenderer.sharedMesh` when loading mod bundles but then treats the result as a plain `Mesh` for ECS — **bone weights ride on the `Mesh` asset**, not on the SMR component at runtime.
- **Pack intake policy** (`intake_rules.yaml`): `unit_model.requires_rigging: true` — content spec already expects rigged infantry.

### 1.2 Inferred runtime model (to confirm with one diagnostic pass in-game)

**Working hypothesis (high confidence):**

1. Vanilla infantry/hero **share a small set of reference rigs** per body class (humanoid infantry, droid, walker, flyer), not one unique skeleton per cosmetic mesh.
2. All units in a class reuse the **same bindpose count and bone name ordering** on their render meshes; swapping cosmetics means swapping **`Mesh` data that remains compatible with that skeleton**.
3. Buildings and vehicles may use static `RenderMesh` meshes (no skinning) — swaps there can remain static.

**Verification checklist (15 min, in gameplay):**

1. Deploy build with diagnostic pass enabled; load a match with mixed vanilla units.
2. Read `BepInEx/dinoforge_debug.log` for vanilla mesh name survey — record `mesh="..."` names per archetype.
3. For one infantry mesh name, inspect whether `Mesh.bindposes.Length > 0`.
4. Compare bindpose counts across two different vanilla infantry types. **If counts match → shared rig retarget is viable.**

---

## 2. Rig + retarget approach for SW meshes

### 2.1 Recommended strategy: **retarget to DINO reference skeleton**

| Option | Verdict |
|--------|---------|
| **A. Retarget to DINO vanilla bindposes** | **Primary** |
| **B. Auto-rig (Rigify / Mixamo)** | Only as step inside A |
| **C. SMR-only swap on prefab** | **Not sufficient** — ECS path never updates scene SMRs |

### 2.2 Per-archetype reference rig library

See `packs/warfare-starwars/assets/rig_reference/README.md`.

### 2.3 Blender pipeline (automated)

Script: `packs/warfare-starwars/assets/tools/blender_rig_and_decimate.py`

1. Import SW static GLB.  
2. Import reference armature; **weight transfer** from reference mesh → SW mesh.  
3. Validate vertex groups + bone count.  
4. **Decimate** to tier budget (infantry 800–2k tris LOD0).  
5. Export `working/<asset_id>/rigged.glb` + validation JSON.

---

## 3. LOD / decimate step (optimizer)

| Tier | LOD0 tris | LOD1 | LOD2 |
|------|-----------|------|------|
| Infantry | 800–2,000 | 60% | 30% |
| Hero | 1,200–3,000 | 60% | 30% |

**Order:** rig/retarget **first**, decimate **second** (avoids breaking weights).

---

## 4. AssetSwapSystem gaps and prototype

### 4.2 Prototype (landed)

**Skinned compatibility gate** in `TrySwapRenderMeshFromBundle`:

- Compare `currentMesh.bindposes.Length` vs `replacementMesh.bindposes.Length`.
- Vanilla skinned + static replacement → **skip swap**, log `#973` hint once per asset.
- Bindpose count mismatch → skip swap.

### 4.3 Future work

- HRV2 mesh swap (#608 P2).  
- Populate `BundleToVanillaMeshMap` after diagnostic survey.  
- `game_verify_mod` check: `bindposes > 0` for infantry bundles.

---

## 5. Phased plan

| Phase | Exit criteria |
|-------|---------------|
| **P0 — Observe** | Reference table in `rig_reference/README.md` |
| **P1 — Rig library** | One humanoid + one droid retarget in Blender |
| **P2 — Art batch** | 3 infantry bundles with `bindposes > 0` |
| **P3 — Bundle pipeline** | PackCompiler emits `SkinnedMeshRenderer` prefabs |
| **P4 — Optimizer** | LOD budgets enforced in CI |
| **P5 — Runtime hardening** | HRV2 path + mesh map + MCP skinning check |

---

## 6. Files touched

| File | Change |
|------|--------|
| `docs/sessions/rigging-optimizer-pipeline-20260531.md` | This document |
| `packs/warfare-starwars/assets/tools/blender_rig_and_decimate.py` | Blender rig + decimate scaffold |
| `packs/warfare-starwars/assets/rig_reference/README.md` | Reference rig library scaffold |
| `src/Runtime/Bridge/AssetSwapSystem.cs` | Skinned mesh compatibility gate |
