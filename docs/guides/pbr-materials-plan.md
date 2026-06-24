# Civis Bevy PBR Materials Integration Plan

**Status:** Phase 1 scaffold landed (code + dirs + asset manifest).
**Owner crate:** `clients/bevy-ref` (`civ-bevy-ref`).
**Feature flag:** `pbr-textures` (off by default; on for daily-driver runs).
**Goal:** Replace the flat per-vertex RGB ("programmer art") in
[`clients/bevy-ref/src/terrain.rs`](../../clients/bevy-ref/src/terrain.rs) with
production-grade PBR-textured ground, blended by biome, with tri-planar cliffs
and decoration meshes.

> Planner note (per `CLAUDE.md`): no implementation code in this doc beyond
> brief pseudocode. Implementation lives in `src/materials.rs` and follow-up
> PRs per phase.

---

## Phase WBS + DAG

| Phase | Task ID | Description | Depends On |
|------:|---------|-------------|------------|
| 1 | P1-T1 | Asset directory skeleton under `assets/textures/<biome>/` | — |
| 1 | P1-T2 | Asset source manifest (`docs/guides/asset-sources.md`) | — |
| 1 | P1-T3 | `materials.rs` module: `Biome` enum, `BiomeMaterials` resource, startup loader (albedo + normal) | P1-T1 |
| 1 | P1-T4 | `pbr-textures` cargo feature wiring; KTX2 image format feature | P1-T3 |
| 1 | P1-T5 | Asset-fetch script (`Tools/fetch-pbr-textures.ps1`) — Poly Haven + ambientCG CC0 downloads, KTX2 packing | P1-T2 |
| 1 | P1-T6 | Switch `terrain.rs` mesh from `Mesh::ATTRIBUTE_COLOR` to UV0; emit per-vertex `biome_id` attribute for splat | P1-T3 |
| 2 | P2-T1 | Pack ORM (AO/Rough/Metal) per biome; extend `load_biome_materials` to bind `metallic_roughness_texture` + `occlusion_texture` | P1-T3, P1-T5 |
| 2 | P2-T2 | Emissive layer hook (e.g. lava biome stub) | P2-T1 |
| 2 | P2-T3 | Per-biome material tuning pass (roughness/reflectance + UV scale) | P2-T1 |
| 3 | P3-T1 | Tri-planar projection shader for cliffs (`rock_cliff`) — avoids stretched UVs on steep slopes | P2-T1 |
| 3 | P3-T2 | Slope-aware biome blend (grass→rock by surface normal Y) | P3-T1, P1-T6 |
| 4 | P4-T1 | Terrain splat-map: per-vertex biome weights → fragment shader blends 4 nearest layers | P1-T6, P2-T1 |
| 4 | P4-T2 | Author splat-map material extending `StandardMaterial` (Bevy `MaterialExtension`) | P4-T1 |
| 4 | P4-T3 | Authoring tool: precompute biome weights from heightmap into a 2D splat texture | P4-T1 |
| 4 | P4-T4 | Reference comparison: drop `bevy_terrain` as alternative path; benchmark vs in-tree impl | P4-T2 |

**DAG summary:**

- `P1-T1 → P1-T3 → P1-T4 → P1-T6 → P2-T1 → P3-T1 → P3-T2`
- `P1-T2 → P1-T5 → P2-T1`
- `P2-T1 → P2-T2`, `P2-T1 → P2-T3`
- `P1-T6 + P2-T1 → P4-T1 → P4-T2 → P4-T3; P4-T2 → P4-T4`

---

## Phase 1 — Albedo + normal per biome (minimum viable PBR)

**Deliverable:** Six `StandardMaterial` handles loaded at startup, each bound
to a CC0 albedo and normal map. Terrain mesh quads pick a biome per triangle
from heightband, render with `BiomeMaterials.handle(biome)`.

**Acceptance:**

- `cargo run -p civ-bevy-ref --features bevy,pbr-textures --bin civ-bevy-window`
  renders six visually distinct textured bands across the heightmap.
- Default `cargo check -p civ-bevy-ref --features bevy` (no `pbr-textures`)
  still builds without asset files present.
- Unit tests in `materials.rs` cover `Biome::ALL`, `index`, `slug`, and
  `from_height_norm` band boundaries.

**Files touched:**

- `clients/bevy-ref/src/materials.rs` (new — landed)
- `clients/bevy-ref/src/lib.rs` (add `pub mod materials;` behind feature)
- `clients/bevy-ref/Cargo.toml` (add `pbr-textures` feature + `bevy/ktx2`)
- `clients/bevy-ref/src/terrain.rs` (emit UV0 + biome attribute)
- `clients/bevy-ref/src/bevy_render.rs` (apply per-biome material to mesh chunks)

**Risk callouts:**

- KTX2 + BasisU support in Bevy 0.18 requires the `bevy/ktx2` and
  `bevy/zstd` features. Confirm at feature wiring time.
- Asset paths use forward slashes inside the Bevy `AssetServer` regardless of
  host OS — Windows-safe.

---

## Phase 2 — Full PBR (ORM + emissive)

**Deliverable:** Each biome material binds `metallic_roughness_texture` and
`occlusion_texture` from a single packed `orm.ktx2` (R=AO, G=Roughness,
B=Metallic), plus optional emissive for special biomes.

**Acceptance:**

- Roughness response visible under directional light sweep (low-angle sun).
- AO visibly darkens crevices on the cliff biome.
- Memory budget: total texture VRAM ≤ 64 MB at 1024² per map (six biomes ×
  three maps × ~1.3 MB BC7 ≈ 24 MB).

**Files touched:**

- `clients/bevy-ref/src/materials.rs` (extend the `StandardMaterial` build to
  bind ORM + emissive)
- `Tools/fetch-pbr-textures.ps1` (channel-pack AO/Rough/Metal PNGs into KTX2)

---

## Phase 3 — Tri-planar projection for cliffs

**Deliverable:** Cliff biome samples albedo/normal/ORM along X, Y, Z world
axes and blends by `abs(normal)` weights, eliminating the stretched-UV look on
steep slopes. Slope-aware blend automatically transitions grass → rock as the
surface normal rotates off vertical.

**Acceptance:**

- No visible UV stretching on slopes > 45° in the default heightmap.
- Smooth transition band between grass/rock without obvious seams.

**Files touched:**

- New WGSL shader: `clients/bevy-ref/assets/shaders/triplanar.wgsl`
- `clients/bevy-ref/src/materials.rs` — wire a custom `Material` /
  `MaterialExtension` for the cliff biome.

**Reference:** `bevy_terrain` (<https://github.com/kurtkuehnert/bevy_terrain>)
ships a tri-planar implementation worth reading — do not vendor wholesale;
extract just the projection helpers.

---

## Phase 4 — Terrain splat-map shader

**Deliverable:** A single `MaterialExtension` over `StandardMaterial` reads a
2D splat texture (RG16 or RGBA8, one channel per dominant biome at that XZ)
and blends up to four nearest biome layers per fragment. Per-vertex biome
weights from Phase 1 become the LOD-far fallback; the splat texture provides
sub-quad detail.

**Acceptance:**

- Smooth biome transitions on the heightmap, no per-quad colour banding.
- Splat authoring is deterministic from heightmap + climate seeds (regression
  test seeds reproducible).
- Performance: ≥ 60 FPS at 1080p on RTX 3090 Ti with the 256² heightmap.

**Files touched:**

- New shader: `clients/bevy-ref/assets/shaders/terrain_splat.wgsl`
- `clients/bevy-ref/src/materials.rs` — register the splat material.
- `clients/bevy-ref/src/terrain.rs` — emit splat texture from heightmap.
- Bench: `clients/bevy-ref/benches/terrain_splat.rs` (criterion).

---

## Cross-Project Reuse Opportunities

Per the Phenotype Org Cross-Project Reuse Protocol:

| Candidate                            | Target shared crate          | Impacted repos               |
|--------------------------------------|------------------------------|------------------------------|
| `Biome` enum + height-band mapping   | `civ-voxel` (new `biome` mod)| Civis, WorldSphereMod3D      |
| KTX2 channel-pack PowerShell script  | `Tools/` (org-wide)          | Civis, DINOForge, WSM3D      |
| Tri-planar WGSL helper functions     | new `phenotype-shaders` crate| Civis, WSM3D, phenotype-voxel|
| CC0 asset manifest schema            | `phenotype-assets` (new)     | All Phenotype game repos     |

Confirmation required from user before any extraction to a sibling crate.

---

## Related docs

- Asset sources: [`asset-sources.md`](./asset-sources.md)
- Texture conventions: [`../../clients/bevy-ref/assets/textures/README.md`](../../clients/bevy-ref/assets/textures/README.md)
- Bevy renderer entry: `clients/bevy-ref/src/bevy_render.rs`
- Terrain mesh: `clients/bevy-ref/src/terrain.rs`
