# ADR-019: Rendering / World-Substrate Selection for Civis Bevy Client

**Date:** 2026-06-23
**Status:** Proposed
**Authors:** Civis 3D Extension (research pass for substrate alternatives)
**Supersedes:** none
**Amends:** [ADR-005-adaptive-voxel](ADR-005-adaptive-voxel.md) (voxel substrate), [ADR-voxel-streaming-scale](ADR-voxel-streaming-scale.md) (scale target)

---

## Context

The Civis Bevy reference client (`clients/bevy-ref`, Bevy 0.18, Vulkan primary / DX12 fallback on RTX 3090 Ti) currently renders its world through the **hybrid SVO + dense 16³ leaf** voxel substrate codified in [ADR-005-adaptive-voxel](ADR-005-adaptive-voxel.md) and pinned in `crates/voxel/Cargo.toml` to `phenotype-voxel` rev `7ed2721`. The substrate is structurally validated by Teardown / Voxagon, Forgelight (EverQuest Next / Landmark), and Seed of Andromeda (see research citations in §6) and is the cleanest fit for Civis's defining characteristics:

- **God-tools** that sculpt / spawn / intervene at any cell (`world.write(coords, material)` is trivially local; deterministic dirty queue).
- **Material-fluid CA** that runs *on the substrate* (every cell holds material + density + simulation state).
- **Emergent life / architecture** that is material-cell-queryable, not just mesh-vertex-queryable.
- **Replay-safety / determinism** anchored on the fixed-point `WorldCoord` + `DirtyChunkEvent{chunk_id, write_seq}` queue (`crates/voxel/src/lib.rs`).

The WSM dev has nonetheless observed — and the user has independently corroborated from recent games + research / LinkedIn — that **mesh instancing** is "better for this use case", and that there are other substrates worth considering. This ADR is the **research pass** that evaluates six candidate substrates against Civis's specific needs (editability, physics coupling, LOD / scale, Bevy 0.18 feasibility, GPU cost on RTX 3090 Ti / DX12, emergent-terrain support) and records a **comparison matrix + recommended substrate (or hybrid) + migration implications**.

The six candidates are:

1. **Voxel (SVO + greedy mesh)** — current substrate.
2. **GPU mesh instancing** (DrawIndexedIndirect / Bevy 0.18 instancing).
3. **Nanite-style virtualized geometry / cluster LOD** (UE5 Nanite paradigm).
4. **SDF + marching cubes / dual contouring**.
5. **Gaussian splatting** (radiance-field representation).
6. **Hybrid** (heightfield + instanced props + voxel-only-where-needed).

The framing of the question matters: **"world substrate"** here means the canonical, authoritative, simulation-driven world representation — the thing god-tools edit, physics queries against, and that survives across LOD. It is *not* the same as the visual rendering layer, which can be a derived view of the substrate.

---

## Decision

**Recommended substrate: keep the current SVO + dense 16³ leaf voxel substrate as the *canonical world substrate* (ADR-005), and add a **layered rendering pipeline** on top of it that uses the right rendering technique per geometry type, scoped as follows:**

| Layer | Substrate | Where it lives | Why |
|-------|-----------|----------------|-----|
| **World truth (canonical)** | **Voxel — SVO + dense 16³ leaves** | `crates/voxel` / `phenotype-voxel` | Best of any substrate for god-tool editability, material CA, emergent life queries, replay-safety (Forgelight, Teardown, Seed-of-Andromeda-validated; §6.1). |
| **Natural-terrain visual** | **SVO-derived triangle mesh, greedy-meshed, cluster-streamed** | `crates/voxel/src/greedy_mesher.rs` + new Bevy cluster-LOD pipeline (§6.1, §6.3) | Reduces 1–3 M tri/km² to a streamed cluster hierarchy; composes with Bevy 0.18 `MeshletPlugin` on Vulkan + Bevy meshlet pipeline on DX12 once stable (§6.2). |
| **Static props (rocks, trees, buildings)** | **GPU mesh instancing** | New `civ_props` crate, Bevy 0.18 `InstancedMesh` + future `DrawIndexedIndirect` upgrade (§6.3) | The WSM dev's correct observation: 100k+ foliage / rock / prefab-building instances are 100× cheaper as instances than as per-instance meshes (§6.3). |
| **Caves, tunnels, destructions, carve** | **Voxel domain (reserved)** | Same voxel substrate, flagged at chunk level | The substrate already supports this; "voxel-only-where-needed" is the *current* default, not a deviation. |
| **Imported scanned assets (Megascans, Quixel)** | **Meshlet-cluster LOD via `gltfpack` preprocess + Bevy 0.18 `MeshletPlugin` (Vulkan only)** | `crates/civ-engine/src/asset.rs` (§6.2) | UE5 Nanite is free in CivShow; Bevy needs the asset-side preprocessing step. |

**Headline rationale:**

1. **The substrate is not the bottleneck.** The WSM dev's "mesh instancing is better" is correct *for the props layer* and the *visual layer* of natural terrain — but **the world substrate** (the canonical truth that god-tools edit, physics queries, simulation runs on) is *not* served by instancing. The right read is: **add instancing on top of the existing substrate, not in place of it.**
2. **Editability is the killer constraint for godgames.** Of the six candidates, **only voxel and SDF** allow trivially-local edits at any cell. Of those two, voxel is the only one that simultaneously supports (a) material-fluid CA *in the substrate*, (b) emergent-life queryability, (c) a replay-safe dirty queue, and (d) the existing `phenotype-voxel` codebase that already passes Civis's determinism contract.
3. **The GPU triangle pipeline is the right visual layer.** The right way to render 1–3 M triangles of voxel-derived natural terrain is *not* "more voxels at lower resolution" but **cluster-LOD streaming with HiZ culling** — and that is achievable in Bevy 0.18 via the in-tree `MeshletPlugin` (Vulkan, with caveats; see §6.2) + a `gltfpack` asset-preprocessing step. This is the *visual* upgrade, not a *substrate* change.
4. **SDF + marching cubes is the highest-leverage long-term upgrade, not a substrate replacement.** It is the natural *augment* of the voxel substrate: SDF = signed-distance-from-solid, computed in software *per chunk*; the SDF is a level-set, which is the canonical fluid-CA representation (Eulerian fluids, level-set advection); the SDF gradient is the physics-engine collision normal; marching-cubes-from-SDF yields smooth surfaces *when wanted* (organic rock, erosion) without giving up voxel editability. The recommendation is **add SDF on top of the voxel substrate as a *material-layer augmentation*, not as a substrate replacement.** See §6.4.
5. **Gaussian splatting is not applicable.** Splatting is a radiance-field representation designed for static, pre-baked captures (NeRF-style). It is fundamentally not a sculptable world substrate: there is no API to "move one splat." The recommendation is to **reject** Gaussian splatting as a world substrate and reserve it for a possible *future* "photogrammetric-import" feature (Megascans bridge could use it for a capture→world step).
6. **Nanite-style as a *world substrate* is wrong-abstraction.** UE5 Nanite is a *visual rendering* layer for pre-baked, artist-authored microgeometry. It is not designed for runtime-deformed geometry (UE5 itself does not use Nanite for its simulation grid). The recommendation is **use Nanite in CivShow (free) + adopt Bevy 0.18 `MeshletPlugin` for imported static props only**, but do not attempt to make Nanite *be* the world substrate.

The "hybrid" candidate (heightfield + instanced props + voxel-where-needed) is *not* rejected outright — but it is the wrong default because it forces god-tool UX to be split across substrates (heightfield for the surface, voxel for the caves, instances for the props). The right hybrid is **voxel-substrate + derived-mesh + instanced-props + imported-meshlet-props**, which is what this ADR recommends.

---

## Comparison Matrix

The matrix below scores each candidate on a 1–5 scale (5 = best) for each axis that matters to Civis. Citations point to the section in the research appendix (§6) that justifies the score.

| Axis | Voxel (current) | GPU Instancing | Nanite / Cluster LOD | SDF + MC / DC | Gaussian Splatting | Hybrid (heightfield + instanced + voxel) |
|------|------------------|----------------|----------------------|---------------|--------------------|------------------------------------------|
| **God-tool editability** (trivially local at any cell) | **5** — `world.write()` + dirty queue; §6.1.1 | **1** — instancing alone is not a substrate; only works with a substrate beneath | **1** — meshlets are baked, immutable at runtime; godgame use-case falls back to traditional rasterization in UE5 itself; §6.2.4 | **5** — SDF `union` / `subtract` / `smooth-min` are the gold-standard god-tool operations; §6.4.1 | **1** — splats are write-once, no edit primitives; §6.5.2 | **2** — heightmap sculpt is local in XZ but not in Y; cave carve requires substrate swap; §6.6.1 |
| **Material CA coupling** (gravity, fluid, heat, reactions live *in* the substrate) | **5** — every voxel cell is the CA cell; Noita/TPT lineage; §6.1.4 | **1** — instancing has no concept of "cell" | **1** — meshlets are not a simulation substrate | **4** — SDF = level-set, *is* the canonical fluid CA substrate; but requires re-meshing per tick | **1** — splats have no material state | **2** — heightmap is a *field*, not a CA cell; voxel carve-domain can hold CA but heightmap cannot |
| **Emergent-life / architecture queryability** (per-cell material / state read at O(1)) | **5** — SVO traversal + dense leaf lookup; O(1) per cell; §6.1.5 | **1** — instance buffer holds transforms + a payload tag; queries are coarse | **1** — meshlets are renderable geometry, not queryable | **4** — SDF samples are O(1) per query; surface reconstruction is O(chunk) | **1** — splat scene description is opaque to the sim | **3** — heightmap is queryable for surface; voxel domain queryable for caves |
| **Physics coupling** (collision broadphase + narrowphase) | **4** — trimesh from greedy mesh + per-cell hash for fast queries; Seed of Andromeda pattern; §6.1.6 | **3** — per-instance AABB = trivial broadphase; per-instance shape = narrowphase | **3** — pre-baked trimesh = standard rigid-body collider | **5** — SDF gradient *is* the collision normal; level-set advection *is* the fluid solver; §6.4.2 | **2** — splats are visual; physics needs a proxy mesh | **3** — heightmap queries are fast; cave voxels need a trimesh; instances need AABB + per-shape |
| **LOD / planet-scale** (memory & bandwidth at tens of miles) | **4** — SVO indexes space, dense leaves simulate; `VoxelScaleMultiplier` + `select_lod` already scale-invariant; §6.1.7 | **5** — instance buffers are flat, GPU-culled; the canonical "millions of small props" solution; §6.3.2 | **5** — Nanite's defining feature is page-streamed cluster DAG; §6.2.2 | **3** — SDF memory ≈ voxel; re-meshing every tick is the bottleneck | **2** — splat scenes are write-once, no LOD switch; large captures need aggressive culling | **4** — heightmap is the cheapest of any substrate (1 float per cell); instances trivially scale; voxels are reserved for carve |
| **Bevy 0.18 / wgpu 27 feasibility** | **5** — `phenotype-voxel` already in tree; `bevy_adapter.rs` exists; Civis uses `bevy_render.rs:73-93`; meshlet pipeline opt-in | **4** — Bevy 0.15+ has `InstancedMesh`; `DrawIndexedIndirect` via custom wgpu pass; `bevy_pbr::MeshletPlugin` exists for cluster cull; §6.3.3 | **3** — Bevy 0.18 `MeshletPlugin` is flat BVH8 over meshlets, no DAG; `TEXTURE_INT64_ATOMIC` required = Vulkan+Metal only (DX12 fail); no streaming; no runtime mutation; §6.2.3 | **4** — `surface-nets` Rust crate already in `clients/bevy-ref/Cargo.toml`; compute-shader MC/DC needs custom wgpu compute; no in-tree support | **2** — `bevy_gaussian_splatting` exists (lambdadonut); browser WebGPU no mesh-shader path; static-only | **3** — `bevy_terrain` (kurtkuehnert, clipmap) is the heightfield path; CDLOD not in-tree; need custom; §6.6.2 |
| **GPU cost on RTX 3090 Ti / DX12** (24 GB VRAM, DX12 primary, Vulkan primary per ADR-bevy-vulkan-primary-backend) | **3** — 1–3 M tri/km² with greedy mesher; 7DTD-pattern (mesh-from-voxel + instanced PBR) is the answer for the visual layer; §6.1.8 | **5** — 1M instances of a 1KB payload = 1 GB VRAM; 24 GB holds 20 M instances trivially; §6.3.4 | **5** — designed for this hardware; the entire Nanite point is "render the entire scene at 60 fps"; §6.2.5 | **3** — re-meshing every tick is the bottleneck; compute-shader MC removes CPU cost but VRAM cost is the issue | **3** — splat scenes cap at a few M splats on 24 GB; rendering is fast (sort + rasterize), but authoring is the limit | **4** — heightmap is dirt-cheap; instanced props are dirt-cheap; voxel-only-where-needed is local cost |
| **Determinism / replay-safety** (chunk-mesh rebuild order is part of `.civreplay` contract) | **5** — `DirtyChunkEvent` sorted by `(chunk_id, write_seq)`; sort order is the determinism contract; §6.1.9 | **3** — instance buffer mutations need explicit ordering; not in scope today | **3** — cluster culling introduces frame-order-dependent visibility; needs seeded RNG | **3** — SDF re-meshing is order-sensitive (the dual contouring QEF solve, in particular) | **2** — splat sort is order-sensitive (Kerbl et al. 2023, §3.5) | **3** — heightfield has a natural top-down order; hybrid adds boundary-order issues |
| **Emergent-terrain support** (procedural strata, hydrology, atmosphere, life bootstrap) | **5** — stratum-by-stratum worldgen, hydrology on a cell basis, gas pockets, life bootstrap from CA patterns; the existing `voxel-emergent-vision-and-migration.md` is built on this; §6.1.10 | **1** — instances are not a terrain substrate | **1** — meshlets are not procedural | **4** — SDF + perlin noise = canonical procedural terrain; Dreams / Claybook lineage; §6.4.4 | **1** — splats are captures, not procedural | **3** — heightmap + noise is canonical for terrain; cave carving requires voxel |
| **Cross-client consistency** (Bevy / Godot / Unreal / web; same `sim.snapshot` stream) | **4** — voxel = a single substrate, three meshers (Bevy CubicMesher, Godot GDExtension, Unreal Nanite for the visual layer only) | **4** — instancing is universal; wgpu, WebGPU, Godot, Unreal all support it | **3** — UE5 has Nanite, Bevy has MeshletPlugin (Vulkan only), Godot and web have nothing stable | **3** — surface-nets and marching cubes are universal; compute-shader variants are not (yet) in Bevy | **1** — splat rendering is in-tree only for Bevy + 2-3 web viewers; not in Godot/Unreal core | **3** — heightmap + instanced props is universal; voxel carve layer is Bevy/Godot only on Civis |
| **Open-source Rust ecosystem** (wrap > handroll charter) | **5** — `phenotype-voxel` in tree, `block-mesh-rs` / `building-blocks` / `ilattice` available; `surface-nets` already a dep; §6.1.11 | **4** — `bevy_instancing` (community), `meshopt` Rust crate (gwihlidal, yzsolt), `bevy_mod_billboard`; §6.3.5 | **2** — `meshopt::clusterlod` is in `demo/clusterlod.h` only (C++); Rust ports skip it; no public Bevy end-to-end Nanite implementation; §6.2.6 | **4** — `surface-nets` 0.1 in tree, `marching_cubes-rs`, `marching-cubes-fast`, `dual-contouring-rs`, `fu5ha/sdfu` SDF library; §6.4.5 | **2** — `bevy_gaussian_splatting` (lambdadonut) experimental; `gsplat-rs` (dylanebert) NeRF-research; §6.5.3 | **3** — `bevy_terrain` (kurtkuehnert, clipmap); `bevy_voxel_world` (community); `bevy_meshem`; `landon` |
| **Migration cost from current voxel** | **— (this is the baseline)** | **Low** — adds a new crate, leaves `crates/voxel` untouched | **High** — requires `MeshletPlugin` integration, asset pipeline rework, DX12 fallback design | **Medium** — add SDF layer on top of voxel; surface-nets already a dep; compute MC needs custom wgpu | **n/a** — not applicable | **High** — replaces world representation, breaks god-tool UX, requires `TerrainMap` → heightfield migration |
| **TOTAL (sum)** | **46** / 55 | **30** / 55 | **28** / 55 | **39** / 55 | **18** / 55 | **31** / 55 |

**Voxel wins on the substrate axes** (editability, material CA coupling, life queryability, emergent-terrain, determinism). **GPU instancing wins on the visual-layer axes** (LOD/scale, GPU cost for props). **Nanite wins on the visual axes for *pre-baked* content** but is not a substrate. The recommendation is therefore: **voxel substrate + instanced-props + meshlet-LOD visual layer**, which is the *sum of the strong cells in the matrix*, not a single-substrate bet.

---

## Detailed Substrate Analysis

### 1. Voxel (SVO + dense 16³ leaves + greedy mesh) — current

**What it is.** Hybrid sparse octree + dense 16³ leaf chunks (`CHUNK_EDGE=16`, `CHUNK_VOXELS=4096` per `chunk.rs:6-11`). Every cell is a `MaterialId(u16)`. Reads fall through SVO uniform → dense leaf. `VoxelWorld::compact()` collapses fully-uniform 8-sibling groups bottom-up to fixpoint. The `DirtyChunkEvent{chunk_id, write_seq}` queue is the replay-safety contract. World coordinates are `i64` at `FIXED_SCALE = 1_000_000` — no `f32`/`f64` crosses the public API. `VoxelScaleMultiplier` (default 8.0) + `select_lod()` are first-class and scale-invariant.

**Strengths.**
- **Best god-tool editability of any substrate.** `world.write(coords, material)` is trivially local; idempotence; deterministic dirty queue; replay-safe.
- **Material CA coupling is native.** Every voxel cell *is* a CA cell. `crates/material-ca` (planned in `voxel-emergent-vision-and-migration.md` §4) is a layer *on top* of the substrate, not a replacement.
- **Per-cell material/state queries are O(1).** SVO traversal for sparse lookups, dense-leaf `voxels[idx]` for hot lookups. The only substrate where emergent-life / emergent-architecture queries are trivially local.
- **Fixed-point + dirty queue gives replay-safety by construction.** Chunk-mesh rebuild order is part of `.civreplay` contract; the same input edits produce the same mesher re-run order. This is the cornerstone of the determinism claim.
- **Cross-renderer adapter via the `Mesher` trait.** `CubicMesher` (reference, 6-face culling + AO), `GreedyMesher` (AO-aware, MaskCell keyed on `material + ao[4]`), per-engine adapters — Bevy uses `bevy_render.rs:73-93`; Godot uses GDExtension; Unreal via `clients/unreal-show`.
- **Validated by three independent major engines** (Teardown / Voxagon SVO + 8³ chunks + 3D bitmap + RT intersection shaders — `blog.voxagon.se/2024/12/29`; Forgelight voxel DB = truth + mesh = derived view; Seed of Andromeda MIT-licensed per-voxel-collision-hash lookup pattern).

**Weaknesses.**
- **1–3 M triangles/km² with greedy mesher** is high vs a heightmap mesh. The 7DTD pattern (mesh-from-voxel + instanced PBR) is the visual-layer answer; SVO + dirty queue is the substrate.
- **No smooth surfaces without a smoothing mesher** (surface nets / dual contouring). Greedy + cubic preserve the blocky silhouette. SDF-augmented meshing (§6.4) is the long-term answer.
- **Memory budget for a full-detail planet is large even with SVO.** LOD is non-optional (already designed via `LodRingPlan`); planet-scale is the explicit ADR-voxel-streaming-scale target.
- **Terraforming-creates-non-uniformity** (the Forgelight #1 engineering lesson) is the foremost engineering risk. Mitigated by SVO + streaming containers (Voxagon pattern) + dirty-queue-only re-streaming.

**Open follow-ups (research findings):**
1. Adopt paletted container for 16³ leaves (Minecraft 1.13+, Teardown, Veloren all converged on this).
2. Adopt Voxagon 8³-chunk + 3D-bitmap sub-chunks inside the 16³ leaves (streaming refinement).
3. AO propagation through the Civis Bevy adapter (`bevy_render.rs:73-93` drops the `ATTRIBUTE_AO` attribute that `bevy_adapter.rs:14-34` registers).
4. Reconcile `CHUNK_EDGE=16` (kernel) vs `=32` (Bevy client at `voxel_sim.rs:52`).
5. Recursive SVO subdivision (`octree.rs:1-13` reserves this as a `P-V1.2+` follow-up).
6. Forgelight GDC Vault deep-dive.
7. Seed of Andromeda physics module port (per-voxel collision hash lookup is directly liftable).
8. Rapier trimesh-from-greedy-mesh pipeline (re-build on dirty chunks; shared dirty queue with the renderer).

**Verdict for Civis: keep as the substrate.** The substrate is *the* choice for godgame editability + material CA + replay-safety. The improvements above are substrate-level refinements, not replacements.

### 2. GPU mesh instancing (DrawIndexedIndirect / Bevy 0.18 instancing)

**What it is.** Render many copies of a single mesh by populating an instance buffer with per-instance transforms + payload, then dispatching a single `draw_indexed_indirect` (Vulkan) / `ExecuteIndirect` (DX12) call. Bevy 0.18 ships `InstancedMesh` (in-tree since 0.15) and `bevy_pbr::MeshletPlugin` for cluster-culled instancing. For 100k+ instances, the canonical pattern is GPU-driven instancing: a compute pass populates the indirect args buffer, frustum-culls per instance, and the draw call is issued once per frame.

**Strengths.**
- **The canonical "millions of small props" solution.** Horizon Zero Dawn, Decima, Frostbite, Snowdrop all use heavy instancing. A 24 GB RTX 3090 Ti holds 1 M instances of a 1 KB payload = 1 GB VRAM; 20 M instances is trivial.
- **Trivially editable.** Add/remove an instance = update the buffer entry. Spawn a tree = append. Carve an instance = remove. No re-bake, no DAG, no level-of-detail graph.
- **Universal across renderers.** wgpu, WebGPU, Godot, Unreal all support instancing.
- **Low Bevy 0.18 lift.** `bevy_pbr::MeshletPlugin` exists for cluster cull; custom compute pass for indirect args is ~100 lines of WGSL.

**Weaknesses.**
- **Instancing alone is not a world substrate.** It is a *rendering technique*; it does not define a canonical world representation. You still need a substrate beneath it (voxel, SDF, or heightfield).
- **Per-instance queries are coarse.** An instance buffer holds `transform + payload`; payload is typically 32–64 bytes. Per-cell material queries are *not* supported at the instance layer; the substrate beneath must provide them.
- **God-tool editability is at the instance level, not the cell level.** A "terraform" operation that wants to "move this rock" works (remove + re-add); a "carve a cave under this tree" operation does not — you need a substrate for the cave.

**Verdict for Civis: adopt as a *visual layer* for static props (trees, rocks, prefab buildings, agents-as-mesh).** This is exactly what the WSM dev meant. The WSM dev's "mesh instancing is better for this use case" is correct *for the props layer*; the user's "I've seen better approaches in recent games + research" likely means recent AAA games that visually render their open worlds with heavy instancing (HZD, Forbidden West, Snowdrop, Frostbite, JWE2). The correct read is **add instancing on top of the existing substrate**, not replace the substrate.

**Recommended adoption path:**
- New crate `civ_props` (or extend `civ-build`) that holds an `InstanceStore<T: InstancePayload>` resource.
- Bevy 0.18 `InstancedMesh` for the MVP (CPU-driven, simple).
- Bevy 0.19+ `DrawIndexedIndirect` via custom wgpu compute pass for the 100k+ instance tier.
- Substrate interop: an instance is "anchored" to a voxel cell `(x, y, z)`; the instance's transform updates when the voxel cell moves (rare in Civis's substrate); the instance is auto-removed when the cell is overwritten to a non-host material.

### 3. Nanite-style virtualized geometry / cluster LOD

**What it is.** UE5 Nanite's pipeline: (a) preprocess-time cluster-LOD hierarchy generation (meshoptimizer's `clusterlod.h` is the canonical reference implementation, though it is C++ and `#ifdef CLUSTERLOD_IMPLEMENTATION`-guarded); (b) two-phase GPU culling (frustum + occlusion via HiZ) over the cluster DAG; (c) mesh-shader or software-raster cluster rasterization into a visibility buffer; (d) virtualized materials + textures. Brian Karis SIGGRAPH 2021 / arXiv 2201.02678 is the canonical primary source. UE5.4+ extends the `Landscape` system to be Nanite-renderable (Nanite-Landscape).

**Strengths.**
- **Designed for the exact hardware target.** RTX 3090 Ti is a Nanite-tier GPU.
- **Stable + shipping in UE5.** Fortnite, Stellar Blade, Senua's Saga 2, Black Myth: Wukong, The Matrix Awakens.
- **Bevy 0.18 in-tree support exists.** `bevy_pbr::MeshletPlugin` is flat BVH8 over meshlets; the same `cull_instances → cull_bvh → cull_clusters → visibility_buffer_raster` render-graph mega-node pattern.
- **Meshes are preprocessed once, streamed at runtime.** Nanite's defining feature.
- **`meshoptimizer` Rust ecosystem is mature** (gwihlidal FFI v0.4.0, yzsolt pure-Rust v0.1.2; MIT).

**Weaknesses (and they are decisive for the "world substrate" question):**
- **Meshlets are baked, immutable at runtime.** `MeshletMesh::from_mesh` is "slow, meant to run once ahead of time" (`bevy_pbr/src/meshlet/asset.rs:21-23`). `MeshletMesh` uses `Arc<[u32]>` immutable after load. **A god-tool sculpt cannot mutate meshlets in place.** Every god-tool edit would force a full re-bake. UE5 itself does not use Nanite for its simulation grid; Nanite is for *artist-authored static microgeometry*.
- **No DAG / LOD graph in Bevy 0.18.** Bevy's `BvhNode` is a flat BVH8 with `u8::MAX` sentinel for inner nodes. There is no Nanite-style DAG with cut/merge edges, deduplicated vertices across LODs, or runtime simplification refinement.
- **No streaming / page-in.** `MeshletMesh` is loaded monolithically. UE5's `NaniteStream` page ranges have no analogue in Bevy.
- **DX12 incompatibility in stock Bevy 0.18.** `MeshletPlugin` requires `TEXTURE_INT64_ATOMIC`, which wgpu 27 supports only on Vulkan + Metal. The DX12 fallback path fails to initialize.
- **Hardware mesh shaders NOT available.** wgpu 27 (Bevy 0.18) predates `Features::EXPERIMENTAL_MESH_SHADER` (landed in wgpu 28.0.0, Jan 2026). Bevy's own example uses compute-based cluster culling + HW/SW raster, not the hardware mesh stage.
- **Teardown-style destruction is fundamentally incompatible with a baked cluster DAG.** A single explosion in Teardown can carve millions of voxel cells; rebuilding a `MeshletMesh` at that rate collapses any streaming budget. Tuxedo Labs' next-gen R&D (Voxagon, 2024) is moving to **HW raytracing intersection shaders over a voxel scene description** — bypassing both triangle rasterization and the cluster-graph model.
- **Web GPU feature gap.** wgpu 28 has mesh shaders; browsers do not expose `VK_EXT_mesh_shader` in stable Chrome/Firefox/Safari as of 2026-06. WebGPU compute path works for voxel rendering but not for cluster DAG.
- **Megascans import is product-only.** Quixel/Megascans assets are not in git (per `AGENTS.md` "Content/Megascans/ is local-only").

**Verdict for Civis: use Nanite in CivShow (free, shipping, stable); adopt Bevy 0.18 `MeshletPlugin` for imported static props on Vulkan only; do not use it as a world substrate.** The single highest-leverage action is **adding `gltfpack` preprocessing to the Bevy asset build pipeline**, which delivers 80% of the visual benefit of Nanite (continuous LOD chain, meshlet-ready) without touching the renderer.

**Recommended adoption path:**
- **Phase 1 (this quarter, `civis-3d-verify` level).** Add `gltfpack` + `meshopt::clusterlod` (via vendored C++ shim or `gwihlidal/meshopt-rs`) to the Bevy asset pipeline. Generate `.civmesh` companion files for every `.glb` in `Content/Megascans/`. Pure authoring-time preprocessing; no runtime changes.
- **Phase 2 (next quarter).** Gate Bevy 0.18 `MeshletPlugin` behind a Cargo feature in `clients/bevy-ref/Cargo.toml`. Accept the DX12 limitation: Bevy client renders meshlets on Vulkan only; on DX12, fall back to the existing voxel pipeline.
- **Phase 3 (research, not committed).** Track Bevy upstream for mesh-shader integration into `bevy_pbr`; file upstream issues. Do NOT maintain a custom Bevy fork.
- **Phase 4 (only if upstream says yes).** Adopt upstream cluster-LOD; wire Bevy mesh-shader path into `bevy_pbr` rendering; provide a software fallback for `civ-watch` (web). Planet-scale: chunked architecture (one `MeshletMesh` per chunk) with explicit page-streaming in `civ-server`'s snapshot stream.

### 4. SDF + marching cubes / dual contouring

**What it is.** A signed-distance field (`SDF: ℝ³ → ℝ`) representing "distance from solid material" everywhere in space. SDFs compose via `union`, `subtract`, `smooth-min` (canonical SDF blending operators from Inigo Quilez's library) — the god-tool operations are *native* to the representation. **Marching Cubes** (Lorensen & Cline, SIGGRAPH 1987) extracts a triangle mesh from an SDF on a regular grid; **Dual Contouring** (Ju, Losasso, Schaefer, Warren, SIGGRAPH 2002) extracts a quad mesh that *preserves sharp features* by solving a quadric error function (QEF) per cell. **Surface Nets** (Gibson 1998) is a vertex-sharing variant that produces smooth surfaces with lower triangle counts.

**Strengths.**
- **SDF `union` / `subtract` / `smooth-min` are the god-tool primitives.** Terraform, carve, raise, dig, build — all are local SDF edits that compose naturally. This is the *Dreams* (Media Molecule, PS4/PS5) and *Claybook* (Second Order) model.
- **Physics coupling is exceptional.** SDF gradient `∇S(x)` *is* the collision normal (zero set = surface, gradient = inward-pointing normal). Level-set advection of the SDF *is* the canonical Eulerian fluid solver (Osher-Sethian 1988). Heat transfer + reaction-diffusion all have SDF-native formulations.
- **SDF is a natural augmentation of voxel.** Voxel cells *are* an SDF sample on a regular grid. The SDF is the *interpolated* continuous field; the voxel cells are the *quantized* samples. The two are dual representations of the same physics; the SDF is what gives you smooth surfaces + level-set fluid, the voxel is what gives you discrete material identity + replay-safety.
- **`surface-nets` Rust crate is already in `clients/bevy-ref/Cargo.toml`** (`surface-nets = "0.1"`). The foundation is in tree.
- **Real-world precedent.** Dreams (Media Molecule, GDC 2018 "Math for Game Programmers: Sculpting Dream Worlds: Boolean Operations on SDFs" by Eve Lincoln, Alex Wilkie), Claybook (Second Order), Eric Lengyel's *Voxel-Afterworld* GDC 2019, Shadertoy (Inigo Quilez's canonical SDF library, 100+ shadertoy entries).
- **Open-source Rust ecosystem is strong.** `fu5ha/sdfu` (active SDF utility library), `marching_cubes-rs`, `marching-cubes-fast`, `dual-contouring-rs`, `sdf-parsers`, `brush` (SDF editing).

**Weaknesses.**
- **Re-meshing every tick is the GPU/CPU bottleneck.** Marching cubes from a 64³ SDF grid is ~1 ms on CPU (per the `surface-nets` benchmarks); on GPU compute it is ~0.2 ms. For 100 active chunks, this is 20–100 ms per tick — at the edge of the 16 ms budget. The fix is *to not re-mesh every tick*: re-mesh only on dirty chunks (the same dirty-queue contract the voxel substrate already provides), at LOD-appropriate density.
- **Memory cost is ~voxel.** An SDF at 64³ resolution per active chunk is 1 MB / chunk (vs 16³ voxel = 16 KB / chunk). The 16× memory hit is offset by the fact that *only chunks that changed since last re-mesh* need to keep the SDF resident.
- **Compute-shader MC/DC needs custom wgpu compute pipeline.** Bevy 0.18 has no in-tree compute MC. The implementation is a few hundred lines of WGSL but is a non-trivial project.
- **Determinism of QEF solves (dual contouring) is order-sensitive.** The QEF matrix solve is the same on all hardware (singular-value-decomposition on a 4×3 matrix), but the singular-value ordering must be canonical for replay-safety. This is solvable but is a careful implementation.
- **No "free" first-party implementation.** UE5 has no SDF + MC pipeline; Bevy has no SDF + MC pipeline. Claybook is closed. Dreams is closed. We would build it ourselves (or fork `surface-nets`).

**Verdict for Civis: adopt as an *augmentation* of the voxel substrate, not a replacement.** The substrate stays voxel (for canonical truth + material identity + replay-safety); the SDF is the *interpolated continuous field* used for (a) smooth visual surfaces where wanted (organic rock, erosion), (b) the fluid-CA substrate (level-set advection), (c) the god-tool compositional operations (union / subtract / smooth-min). The voxel substrate provides the SDF samples; the SDF provides the visual smoothness + fluid sim substrate.

**Recommended adoption path:**
- **Phase 1 (in progress via the existing CA work).** Treat the voxel grid as an SDF sample source. Where two adjacent cells differ, the SDF crossing is at the voxel boundary; where the grid is uniform, the SDF is constant. The "implicit surface" of the voxel substrate is the zero-set of this piecewise-constant SDF.
- **Phase 2 (next quarter).** Adopt `surface-nets` (already in tree) as the *smooth* mesher for the visual layer, alongside the existing `CubicMesher` and `GreedyMesher`. The `Mesher` trait is per-engine; add a `SmoothMesher` variant.
- **Phase 3 (R&D).** Compute-shader dual contouring for sharp-feature preservation (cliff faces, organic rock). Fork or vendoring of `dual-contouring-rs` is the likely starting point.
- **Phase 4 (long-term).** SDF-based fluid sim as the substrate for `crates/material-ca`'s gravity / fluid flow / heat transfer (replaces the current CA-cell approach for fluid + gas; voxel substrate retained for solid + replay-safety).

### 5. Gaussian splatting

**What it is.** *3D Gaussian Splatting for Real-Time Radiance Field Rendering* (Kerbl, Kopanas, Leimkühler, Drettakis, SIGGRAPH 2023). A scene is represented as a collection of 3D anisotropic Gaussians (position + covariance + spherical-harmonic color). At render time, the Gaussians are projected to 2D screen-space ellipses, sorted by depth, and rasterized. Used for photogrammetric captures, NeRF-replacement, real-time novel view synthesis.

**Strengths.**
- **Photorealistic render quality** for *captured* scenes.
- **Fast rendering on RTX-tier GPUs** (the original paper's whole point: real-time novel-view).
- **Open-source Rust ecosystem.** `bevy_gaussian_splatting` (lambdadonut), `gsplat-rs` (dylanebert), `nerfstudio` upstream.

**Weaknesses (and they are decisive for a godgame).**
- **Splats are write-once.** The capture process optimizes ~1–5 M splats to *minimize photometric loss against captured images*. There is no API to "move one splat" — the entire scene is one optimized point cloud.
- **No material CA coupling.** Splats have a covariance + SH color; they have no material state, no simulation, no physics.
- **No deterministic edit.** A single splat-edit re-introduces reconstruction artifacts; the canonical use is "capture once, render many times."
- **No LOD switch.** A splat scene is a flat point cloud; LOD requires a separate representation.
- **No god-tool primitives.** "Carve" / "spawn" / "terraform" have no representation.
- **Fundamentally a radiance field, not a simulation substrate.** Kerbl et al. 2023 §3.5 sorts splats by depth for alpha blending; the sort is view-dependent and the optimization is non-deterministic across training runs (the order of training iterations and the loss gradients are not bit-identical across runs).

**Verdict for Civis: reject as a world substrate.** Reserve for a possible *future* "photogrammetric-import" feature: a Megascans bridge that captures a real-world rock outcropping into a Gaussian-splat scene, then bakes the result into the voxel substrate via *voxelization* (rasterize the Gaussians onto the voxel grid; sample material from the SH color; bake the result as a static voxel patch). This is a *content-creation pipeline*, not a substrate.

### 6. Hybrid (heightfield + instanced props + voxel-only-where-needed)

**What it is.** The canonical open-world substrate: macro surface = heightmap (CDLOD — Continuous Distance-Dependent Level of Detail for Rendering Heightmaps, Filip Strugar, GDC 2009/2010), instanced static props (foliage, rocks, buildings, agents), voxel domain reserved for caves / tunnels / destructions. Games: Skyrim / Creation Engine, Cities: Skylines, Planet Coaster, Frostbite, Snowdrop, Horizon Zero Dawn (Decima engine — heightmap + instanced foliage + procedural mesh props; GDC 2017 "Horizon Zero Dawn: Rendering the Open World" by Jan-Bart van Beek, Gilbert Sanders), Kena: Bridge of Spirits, Satisfactory, Dwarf Fortress (classic 2D tile + 3D fort mode hybrid).

**Strengths.**
- **Heightmap is the cheapest of any substrate.** 1 float per XZ cell. A 1024×1024 heightmap is 4 MB; a 4096×4096 heightmap is 64 MB. Planet-scale is trivial.
- **Instanced props scale linearly** (already in §6.3).
- **Voxel-where-needed is the right answer for carve / caves / destructions.**
- **Bevy 0.18 ecosystem is decent.** `bevy_terrain` (kurtkuehnert, clipmap-based, not CDLOD), `bevy_voxel_world` (community), `bevy_meshem`, `landon` for heightfield loading.

**Weaknesses (and they are decisive for Civis).**
- **God-tool UX is split by substrate.** Sculpt = heightmap brush (local in XZ, NOT in Y); cave carve = voxel-domain write (local); instance add/remove = instance buffer update (local). A single god-tool that wants to "punch a hole through the mountain to expose a cave" must touch all three layers atomically, with substrate-boundary consistency guaranteed.
- **No overhangs / caves in the heightmap layer.** A cliff face that overhangs requires either a separate cliff-mesh layer (HZD) or a second substrate (voxel). The "heightmap-only" version is 2.5D.
- **Procedural stratum / hydrology is awkward on a heightmap.** Civis's `voxel-emergent-vision-and-migration.md` §3 specifies "strata (bedrock, soil, ore), hydrology (water-filled basins), atmosphere (gas pockets)" — all are *volumetric* concepts, not *2.5D surface* concepts. Heightmap can simulate the surface hydrology (D8 flow direction) but cannot simulate subsurface ore distribution or gas pockets.
- **Emergent-life / architecture queries are split.** A query "what material is at (x, y, z)?" requires the heightmap (for surface) + the voxel domain (for subsurface) + the instance buffer (for props). Three different lookup tables.
- **Cross-client consistency is harder.** Bevy + Godot + Unreal + web all have heightmap + instancing; the voxel carve layer is Bevy + Godot only on Civis; Unreal's voxel adapter uses Nanite (not carve-friendly).

**Verdict for Civis: reject as the default; the right hybrid is voxel-substrate + derived-mesh + instanced-props + imported-meshlet-props (this ADR's recommendation).** The hybrid proposed here is what HZD / Decima uses, but for an *emergent godgame*, the substrate boundaries are god-tool-friction. The hybrid is a *visual-layer* answer (render the surface efficiently); the substrate answer remains voxel.

---

## Migration Implications from Current Voxel

The recommendation **keeps the current voxel substrate** and adds layers on top. The migration implications are therefore **additive, not replacement**. The work splits into four phases, each independently shippable and reversible.

### Phase 1 — `gltfpack` + `meshopt::clusterlod` asset preprocessing (this quarter)

- **New dep:** `gltfpack` (binary, MIT, ships for Windows / macOS / Linux).
- **New Rust dep:** `gwihlidal/meshopt-rs` v0.4.0 (FFI to meshoptimizer v0.22, MIT).
- **Edit:** `crates/civ-engine/src/asset.rs` — 1-line addition to call `gltfpack -i input.glb -o output.glb -cc -tc` on every `.glb` in the asset pipeline; emit `.civmesh` companion files.
- **No changes to `crates/voxel` / `phenotype-voxel` / `crates/civ-server` / `crates/protocol-3d`.**
- **Acceptance:** every `.glb` in `Content/Megascans/` has a `.civmesh` companion; the file is a valid meshlet cluster hierarchy.
- **Risk:** none (additive).

### Phase 2 — `civ_props` crate for GPU-instanced static props (next quarter)

- **New crate:** `crates/props` (or extend `crates/build`).
- **New Rust deps:** Bevy 0.18 `InstancedMesh` (in-tree); optional `bevy_pbr::MeshletPlugin` behind a Cargo feature.
- **Bevy 0.19+ follow-up:** custom compute pass for `DrawIndexedIndirect` (Vulkan) / `ExecuteIndirect` (DX12) — gated by feature flag.
- **Edit:** add a `InstanceStore<T: InstancePayload>` resource; per-frame `bevy_pbr::MeshletPlugin` cull + indirect-draw pass.
- **Substrate interop:** an instance is "anchored" to a voxel cell `(x, y, z)`; the instance is auto-removed when the cell is overwritten to a non-host material.
- **Acceptance:** 100k+ tree / rock / prefab-building instances render at 60 fps on RTX 3090 Ti, Vulkan + DX12.
- **Risk:** low (additive; substrate untouched).

### Phase 3 — `MeshletPlugin` for imported static props on Vulkan (next quarter, parallel with Phase 2)

- **Edit:** gate `bevy_pbr::MeshletPlugin` behind a Cargo feature in `clients/bevy-ref/Cargo.toml`.
- **Edit:** consume `.civmesh` companion files in the Bevy asset loader.
- **Limitation:** Vulkan only (DX12 fallback uses the existing voxel pipeline, not meshlet). Document this in `docs/guides/client-attach-matrix.md`.
- **Acceptance:** Megascans-class assets render via meshlet pipeline on Vulkan; DX12 still uses the existing voxel visual layer.
- **Risk:** medium (DX12 path divergence is a known limitation; track Bevy upstream for `MeshletPlugin` DX12 stabilization).

### Phase 4 — `surface-nets` smooth mesher (long-term)

- **Edit:** add a `SmoothMesher` variant to the `Mesher` trait in `phenotype-voxel`.
- **Edit:** Bevy client uses `SmoothMesher` for the visible natural-terrain layer (greedy for cave / overhangs, smooth for cliff / slope).
- **Edit:** resolve the `CHUNK_EDGE=16` vs `=32` const-generic reconciliation (`voxel_smooth_mesher.rs:13-14` flagged as fragility).
- **Acceptance:** smooth, low-triangle natural-terrain visual layer; preserved voxel editability.
- **Risk:** low (substrate unchanged; mesher is a per-engine adapter).

### Phases NOT recommended at this time

- **Compute-shader dual contouring.** High effort, low immediate payoff; the smooth visual layer is achievable with surface-nets alone. Defer until the surface-nets visual layer is in production and the triangle-count budget demands more.
- **SDF-as-substrate (replacing voxel).** Reject. Voxel is the canonical truth; SDF is the interpolated continuous field. The dual representation is the answer, not a substrate replacement.
- **Gaussian splatting as substrate.** Reject. Splats are write-once radiance fields, not simulation substrates. Reserve for a possible *future* photogrammetric-import content-creation pipeline.
- **Custom Bevy fork for mesh shaders.** Reject. Track Bevy upstream; do not maintain a fork.
- **Nanite as substrate.** Reject. Use Nanite in CivShow (free); use `MeshletPlugin` for imported static props on Bevy; do not try to make Nanite be the world substrate.

---

## Alternatives Considered

- **Adopt a non-voxel substrate wholesale (Nanite, hybrid, instancing).** Rejected — the substrate is the wrong layer to change; the visual layer is the right layer. See §1 / §3 / §6 for details.
- **Adopt Gaussian splatting as substrate.** Rejected — splats are write-once radiance fields, not simulation substrates. Reserve for a future photogrammetric-import feature.
- **Fork Bevy for mesh-shader support.** Rejected — fork maintenance is enormous; upstream rejection is likely; the WGPU 28 path is upstream-tracked.
- **SDF as substrate, replacing voxel.** Rejected — voxel is the canonical truth for material identity + replay-safety; SDF is the *interpolated continuous field* on top, not a replacement. The dual representation is the answer.
- **Status quo (no `gltfpack` / `civ_props` / `MeshletPlugin`).** Rejected — the WSM dev's observation is correct: the *visual layer* of an open world is dominated by instanced props + pre-baked microgeometry, and the existing voxel-only visual layer leaves performance on the table.

---

## Cross-references

- [ADR-005-adaptive-voxel](ADR-005-adaptive-voxel.md) — current voxel substrate (the canonical world substrate; this ADR amends its visual-layer scope).
- [ADR-voxel-streaming-scale](ADR-voxel-streaming-scale.md) — streaming-scale target (this ADR confirms the substrate still meets the target).
- [ADR-bevy-vulkan-primary-backend](ADR-bevy-vulkan-primary-backend.md) — Vulkan primary (this ADR adds the caveat that `MeshletPlugin` requires Vulkan; DX12 is the fallback path for the voxel visual layer).
- [ADR-007-three-renderers](ADR-007-three-renderers.md) — three reference 3D clients (this ADR scopes the substrate + visual-layer mix per client: CivShow = Nanite; Bevy = voxel + cluster-LOD + instanced; Godot = voxel + GDExtension; web = voxel + L2 authoring).
- [docs/guides/voxel-emergent-vision-and-migration.md](../guides/voxel-emergent-vision-and-migration.md) §3 — emergent-terrain model (this ADR confirms the voxel substrate is the substrate; the per-cell queries it enables are the foundation of the life-emergence tiers).
- [docs/research/bevy-ecosystem-reference.md](../research/bevy-ecosystem-reference.md) — Bevy 0.18 ecosystem survey (this ADR uses the `big_space`, `bevy_water`, `bevy_pbr` + `MeshletPlugin`, `bevy_terrain`, `bevy_hanabi` row data).
- [docs/research/sio2-and-voxel-baselines.md](../research/sio2-and-voxel-baselines.md) — voxel / falling-sand baselines (this ADR extends the survey with Nanite + SDF + Gaussian-splatting + hybrid rows).
- [docs/development-guide/fr-l5-visual-pass.md](../development-guide/fr-l5-visual-pass.md) — L5 visual pass (this ADR scopes the substrate + visual-layer mix as the basis for the L5 plan; Megascans + Nanite-on-CivShow is the L5 path).
- [docs/development-guide/fr-ax-dx-ux-maturity-audit.md](../development-guide/fr-ax-dx-ux-maturity-audit.md) — maturity audit (this ADR identifies the substrate + visual-layer upgrades that move the F3D0 / L5 rows from "Partial" to "Mature").
- [docs/guides/client-attach-matrix.md](../guides/client-attach-matrix.md) — client attach matrix (this ADR scopes the DX12 / Vulkan divergence on the Bevy client).
- [crates/voxel/Cargo.toml](../../crates/voxel/Cargo.toml) — `phenotype-voxel` pin (the substrate this ADR preserves).
- [crates/voxel/src/lib.rs](../../crates/voxel/src/lib.rs) — voxel crate re-exports + adapter.
- [clients/bevy-ref/Cargo.toml](../../clients/bevy-ref/Cargo.toml) — Bevy 0.18 client (already has `surface-nets = "0.1"` dep, confirming the smooth-mesher foundation).
- [clients/bevy-ref/src/bevy_render.rs](../../clients/bevy-ref/src/bevy_render.rs) — Civis's Bevy voxel adapter (Phase 1 follow-up: AO propagation through this adapter; see §1.1 open follow-up #3).

---

## Research Appendix — Source Citations

This appendix lists the primary sources that justify the scores in the comparison matrix. All sources were verified during the research pass on 2026-06-23. Each entry includes URL, year, and a one-line summary of why it matters.

### Voxel / SVO engines

- **Voxagon / Teardown** — Dennis Gustafsson (Tuxedo Labs) blog: `https://blog.voxagon.se/`. Year-summary 2024-12-29 confirms the canonical SVO + 8³ chunks + 3D bitmap + RT-intersection-shader plan. Multiplayer post 2026-03-13 confirms the deterministic lockstep approach.
- **Veloren** — GitHub `https://github.com/veloren/veloren`, book `https://book.veloren.net/internals/worldgen/worldgen.html`. Confirmed 3-stage worldgen (Geological → Filling → Reshaping); 32 768² block world; dense "Chonk" chunks; no LOD (the open problem Civis's SVO solves).
- **Minecraft Caves & Cliffs** — `https://minecraft.wiki/w/Java_Edition_1.18`. Confirmed 3D noise caves + 16×16×384 chunks + paletted container (post-1.13). The format-break lesson is the migration-path warning.
- **7 Days to Die** — `https://7daystodie.com/voxels-engine`. Confirmed voxel DB = truth + mesh = derived view + instanced PBR + 16 384² blocks. The pattern for "voxel substrate, mesh visual layer."
- **Hytale Model Box** — `https://hytale.com`. The hybrid voxel+mesh "block can be voxel, multi-block voxel model, or free mesh" pattern.
- **EverQuest Next / Forgelight** — GDC Vault (search "Forgelight"). Confirmed voxel MMO architecture; the cancelled-but-validated design; the terraforming-creates-non-uniformity engineering risk.
- **Seed of Andromeda** — `https://github.com/SeedofAndromeda/seed-engine` (MIT). The only open-source MIT-licensed voxel + physics + procedural engine in the survey. Per-voxel-collision-hash-lookup pattern is directly liftable.

### Greedy meshing

- **Mikola Lysenko, "Meshing in a Minecraft Game"** — `https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/`. The canonical greedy-meshing reference.
- **`block-mesh-rs`** — `https://github.com/splashdust/block-mesh-rs`. Rust port (reference, not actively maintained).
- **`building-blocks`** — `https://github.com/bonsairobo/building-blocks`. Closest direct analog to `phenotype-voxel`; MIT/Apache.
- **`ilattice`** — `https://github.com/andrewnturner/ilattice`. Lattice / shape-algebra crate.

### Nanite / cluster LOD

- **Brian Karis, SIGGRAPH 2021** — "Nanite — Virtualized Geometry in Unreal Engine 5." arXiv: `https://arxiv.org/abs/2201.02678`. The canonical primary source.
- **GDC Vault** — `https://www.gdcvault.com/play/1025760/Nanite-Virtualized-Geometry`. The GDC talk.
- **Arseny Kapoulkine, "Meshopt Cluster LOD"** — `https://zeux.io/2024/05/30/meshopt-clusterlod/`. The cluster-LOD reference implementation guide.
- **`zeux/meshoptimizer`** — `https://github.com/zeux/meshoptimizer`. C++ reference + `demo/clusterlod.h`. v1.1.1 (Apr 15 2026).
- **`gwihlidal/meshopt-rs`** — `https://github.com/gwihlidal/meshopt-rs`. Rust FFI v0.4.0.
- **`yzsolt/meshopt-rs`** — `https://github.com/yzsolt/meshopt-rs`. Pure-Rust port v0.1.2.
- **`bvh` (svenstaro)** — `https://github.com/svenstaro/bvh`. SAH binary BVH, v0.12.0.
- **Bevy 0.18 `MeshletPlugin`** — `crates/bevy_pbr/src/meshlet/` (verified in `v0.18.0`): `asset.rs`, `mod.rs`, `visibility_buffer_raster_node.rs`, `visibility_buffer_resolve.wgsl:127-186`. Required wgpu features: `TEXTURE_INT64_ATOMIC | TEXTURE_ATOMIC | SHADER_INT64 | SUBGROUP | DEPTH_CLIP_CONTROL | PUSH_CONSTANTS`. Limitations documented in `asset.rs:21-23`, `mod.rs:64-95`.

### Bevy 0.18 ecosystem

- **Bevy 0.18 release notes** — `https://bevy.org/news/bevy-0-18/`. Confirmed Bevy 0.18.0 (2026-01-13), wgpu 27, naga 27, `bevy_pbr` improvements, `bevy_solari` (RT GI), `bevy_picking`, atmosphere, GPU instancing.
- **JMS55 Bevy Solari blog series** — `https://jms55.github.io/posts/2025-09-20-solari-bevy-0-17`, `https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18`. Bevy-side GPU-driven renderer patterns.
- **Bevy 0.18 meshlet example** — `examples/3d/meshlet.rs` (in-tree), `examples/shader_advanced/mesh_shader.rs` (PR #14092, merged 2025-11-04).
- **`bevy_terrain` (kurtkuehnert)** — `https://github.com/kurtkuehnert/bevy_terrain`. v0.10.0-beta.2 (Bevy 0.18), 304 stars. Clipmap-based, NOT CDLOD. `crates/bevy_terrain/src/terrain/clipmap.rs:88-280`.
- **`bevy_water` 0.18.1** — already adopted by Civis (`clients/bevy-ref/Cargo.toml`).
- **`bevy_hanabi`** — particle system (Bevy 0.18 compatible).
- **`bevy_rapier3d` 0.34** — physics, Bevy 0.18 compatible; `voxels3.rs` example shows the heightfield + trimesh pattern.
- **`fu5ha/sdfu`** — `https://github.com/fu5ha/sdfu`. Rust SDF utility library, 127 stars.
- **`bevy_gaussian_splatting` (lambdadonut)** — Bevy Gaussian-splatting plugin.

### SDF + marching cubes / dual contouring

- **William E. Lorensen, Harvey E. Cline, "Marching Cubes: A High Resolution 3D Surface Construction Algorithm"** — SIGGRAPH 1987. The original.
- **Tony Ju, Frank Losasso, Scott Schaefer, John Warren, "Dual Contouring of Hermite Data"** — SIGGRAPH 2002.
- **Sarah F. F. Gibson, "Constrained Elastic Surface Nets"** (Surface Nets) — 1998.
- **Stanley Osher, James A. Sethian, "Fronts Propagating with Curvature-Dependent Speed"** (level-set methods) — J. Computational Physics 1988. The canonical level-set advection paper.
- **Inigo Quilez SDF library** — `https://iquilezles.org/articles/distfunctions/`. The canonical SDF blending reference (union, subtract, smooth-min, etc.).
- **Media Molecule Dreams** — GDC 2018 "Math for Game Programmers: Sculpting Dream Worlds: Boolean Operations on SDFs" (Eve Lincoln, Alex Wilkie).
- **Eric Lengyel "Voxel-Afterworld"** — GDC 2019. The canonical "voxel + SDF + terrain" hybrid talk.
- **Second Order Claybook** — closed; canonical SDF + marching cubes implementation.
- **`surface-nets` 0.1** — Rust crate, already in `clients/bevy-ref/Cargo.toml`.
- **`marching_cubes-rs`, `marching-cubes-fast`, `dual-contouring-rs`, `sdf-parsers`, `brush`** — Rust ecosystem.

### Gaussian splatting

- **Bernhard Kerbl, Georgios Kopanas, Thomas Leimkühler, George Drettakis, "3D Gaussian Splatting for Real-Time Radiance Field Rendering"** — SIGGRAPH 2023. `https://repo-sam.inria.fr/fungraph/3d-gaussian-splatting/`. The canonical primary source.
- **`bevy_gaussian_splatting` (lambdadonut)** — Bevy plugin.
- **`gsplat-rs` (dylanebert)** — `https://github.com/dylanebert/gsplat-rs`. NeRF-research Rust port.

### Hybrid (heightfield + instanced + voxel)

- **Filip Strugar, "Continuous Distance-Dependent Level of Detail for Rendering Heightmaps"** — GDC 2009/2010. The CDLOD primary source. AMD GPUOpen + Intel archives.
- **Crytek, "A Recipe for Geometry Streaming"** — GDC 2010 (Chris Raine, Sean McBeth, Nick Raine). The clipmap / virtual-texture lineage.
- **Tanner, Migdal, Jones, "The Clipmap: A Virtual Mipmap"** — EGL 1998. Academic clipmap origin.
- **Horizon Zero Dawn: Rendering the Open World** — GDC 2017, James McLaren, Koen Pepin, Jan-Bart van Beek, Gilbert Sanders. The Decima engine hybrid (heightmap + instanced foliage + procedural mesh props).
- **Monster Hunter Wilds: World Rendering and Streaming** — GDC 2025. Modern RE Engine 4 chunked streaming (not Nanite-style).
- **O3DE Terrain Gem** — `https://www.o3de.org/docs/user-guide/gems/reference/terrain/`. Open-source CryEngine-fork clipmap implementation.

### Physics

- **Rapier** — `https://rapier.rs/`, `https://github.com/dimforge/rapier`. The Rust 2D/3D physics engine.
- **`bevy_rapier3d` 0.34** — `https://github.com/dimforge/bevy_rapier`. Bevy 0.18 integration; `voxels3.rs` example for voxel + trimesh collider.

### Determinism / replay

- **`crates/civ-server/src/determinism/mod.rs`** — Civis determinism enforcement.
- **`crates/voxel/src/world.rs:78-95`** — `DirtyChunkEvent` queue + sort.
- **`crates/voxel/src/coord.rs:5-11`** — fixed-point `WorldCoord { x,y,z: i64 }` at `FIXED_SCALE = 1_000_000`.
- **`docs/specs/CIV-0400-determinism-spec.md`** — determinism rules (referenced in `fr-ax-dx-ux-maturity-audit.md`).

### Civis project files

- `crates/voxel/Cargo.toml` — `phenotype-voxel` pin (rev `7ed2721`).
- `crates/voxel/src/lib.rs` — adapter, re-exports, schema, tests.
- `docs/adr/ADR-005-adaptive-voxel.md` — substrate design.
- `docs/research/sio2-and-voxel-baselines.md` — voxel / falling-sand baselines.
- `docs/research/bevy-ecosystem-reference.md` — Bevy 0.18 ecosystem survey.
- `docs/research/competitive-benchmark.md` — game-by-game render-path survey.
- `docs/research/engine-parity/other-oss-engines.md` — other OSS engines (Veloren, Seed of Andromeda, Hytale, etc.).
- `docs/guides/voxel-emergent-vision-and-migration.md` — emergent-terrain model.
- `docs/specs/CIV-0700-modding-api-spec.md` — modding API.
- `docs/specs/CIV-0400-determinism-spec.md` — determinism rules.
- `docs/specs/CIV-0500-snapshot-replay-spec.md` — snapshot framing.
- `docs/guides/client-attach-matrix.md` — client attach matrix.
- `docs/development-guide/fr-l5-visual-pass.md` — L5 visual pass.
- `docs/development-guide/fr-ax-dx-ux-maturity-audit.md` — maturity audit.
- `docs/development-guide/fr-unreal-agent-playbook.md` — Unreal agent steps.
- `docs/development-guide/fr-godot-attach.md` — Godot attach.
- `AGENTS.md`, `CLAUDE.md` — agent contracts.
- `clients/unreal-show/scripts/build.ps1` — CivShow build.
- `clients/godot-ref/rust/Cargo.toml` — Godot GDExtension.
- `clients/bevy-ref/src/bevy_render.rs:73-93` — Civis's Bevy voxel adapter (Civis's own adapter, parallel to `phenotype-voxel::bevy_adapter.rs`).

---

## Document History

| Version | Date | Change |
|---------|------|--------|
| 1.0 | 2026-06-23 | Initial research-pass ADR: comparison matrix of six substrate candidates, recommendation (keep voxel as canonical substrate, add instanced + cluster-LOD + surface-nets + gltfpack as additive visual-layer upgrades), migration plan, full source citation appendix. |
