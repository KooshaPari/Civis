# Terrain PBR + GI Implementation Spec

**Audience:** Execution agent  
**Date:** 2026-05-28 (supersedes prior stub at this path)  
**Status:** Ready for implementation  
**Predecessor docs:** `docs/guides/aaa-quality-roadmap.md`, `docs/research/bevy-gi-status.md`, `docs/guides/pbr-materials-plan.md`

All component names and module paths in this document are verified against
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy*-0.18.1` source on 2026-05-28.

---

## 0. Current-state snapshot

| File | Relevant state |
|------|----------------|
| `clients/bevy-ref/src/terrain.rs` | `terrain_mesh()` emits `ATTRIBUTE_POSITION`, `ATTRIBUTE_NORMAL`, `ATTRIBUTE_COLOR` (sRGB f32x4 per vertex via `color_for_height`). No `ATTRIBUTE_UV_0` is written. Height-band thresholds used by `color_for_height` are the canonical biome splits. |
| `clients/bevy-ref/src/materials.rs` | `BiomeMaterialsPlugin` + `BiomeMaterials` resource fully scaffolded behind `pbr-textures` Cargo feature. Loads `albedo.jpg` + `normal.jpg` per biome at startup. ORM slot present but points at `orm.ktx2` (not yet downloaded). |
| `clients/bevy-ref/Cargo.toml` | `bevy = "0.18"`, `wgpu = "27"`. `solari` feature already wires `bevy/bevy_solari`. `pbr-textures` feature is NOT declared in `[features]` — it exists only in code `#[cfg]` guards; the plugin is unreachable until this is fixed. |
| Assets on disk | `clients/bevy-ref/assets/textures/{grass_field,sand_beach,rock_cliff,snow_pure,forest_floor,dirt_ground}/albedo.jpg` and `normal.jpg` are present (CC0). `orm.ktx2` is absent. |

---

## 1. Terrain splat material

### 1-a. Step 1 — Tiled `StandardMaterial` proof-of-concept (implement first)

**Objective:** Replace solid `color_for_height` vertex shading with real PBR albedo + normal-map textures, using on-disk assets, with no WGSL authoring.

**Required mesh change (agent task in `terrain.rs`):**

`StandardMaterial` with `base_color_texture` samples from `ATTRIBUTE_UV_0`. This attribute is currently absent from `terrain_mesh()`. The agent must add it:

- Compute `uv = [wx / UV_TILE_SCALE, wz / UV_TILE_SCALE]` for each vertex, where `UV_TILE_SCALE = 8.0` (one texture repeat per 8 world units; tune post-integration).
- Push into the mesh as `Mesh::ATTRIBUTE_UV_0` (type `Vec2`).
- Leave `ATTRIBUTE_COLOR` in place — it is harmless and may be read by other passes (wireframe, debug overlay). Do not remove it without auditing consumers.

**Required Cargo feature fix (agent task in `Cargo.toml`):**

Add to `[features]`:
```
pbr-textures = ["bevy"]
```
Then add `BiomeMaterialsPlugin` to `App` in `standalone.rs` under `#[cfg(feature = "pbr-textures")]`.

**Terrain spawn change (agent task in `standalone.rs`):**

Replace the single-material terrain spawn with six `Mesh3d` entities, one per biome height band. Each entity carries `MeshMaterial3d(materials.handle(biome).clone())`. Sub-mesh index ranges per band are computed from `Biome::from_height_norm(height / HEIGHT_SCALE)` during mesh construction — pre-split the `Indices::U32` list by band and emit six index buffers from the same vertex pool.

Alternatively, if mesh splitting adds complexity, use a single terrain mesh with the dominant (grass) material as an interim step, then layer the full six-band split in a follow-on commit.

**`StandardMaterial` field reference (already wired in `load_biome_materials`):**

| Field | Value |
|---|---|
| `base_color` | `Color::srgb(r, g, b)` from `Biome::fallback_srgb()` |
| `base_color_texture` | `asset_server.load("textures/{slug}/albedo.jpg")` |
| `normal_map_texture` | `asset_server.load("textures/{slug}/normal.jpg")` |
| `perceptual_roughness` | `0.95` |
| `metallic` | `0.0` |
| `reflectance` | `0.18` |
| `metallic_roughness_texture` | `None` (Phase 2, awaiting ORM download) |
| `occlusion_texture` | `None` (Phase 2) |

**Vertex attributes needed:** only `ATTRIBUTE_UV_0` (standard Bevy attribute). No custom vertex attribute required for Step 1.

---

### 1-b. Step 2 — Full splat shader via `MaterialExtension` (Phase D, follow-on)

**Objective:** Single terrain mesh with a WGSL fragment shader blending all six biome textures per-fragment by height and slope.

**Architecture:**

Use Bevy 0.18's `ExtendedMaterial<StandardMaterial, TerrainSplatExtension>`. The extension struct holds 12 `Handle<Image>` (six albedo + six normal) and a uniform buffer with the six height thresholds and per-biome roughness scalars. The extension fragment shader reads `world_position.y` and `world_normal.y` from the standard vertex output — no custom vertex attribute is required for pure height/slope blending.

The current `ATTRIBUTE_COLOR` remains in place; the splat shader ignores it.

**Custom vertex attribute (optional, for biome-weight pre-baking):**

If smooth transitions between non-adjacent biomes are needed, a pre-baked weight attribute can be added to `terrain_mesh()`:

```
pub const ATTRIBUTE_BIOME_WEIGHTS: MeshVertexAttribute =
    MeshVertexAttribute::new("BiomeWeights", 1337, VertexFormat::Float32x4);
```

This is optional — pure height/slope blending in the shader is sufficient for Step 2. Do not add this attribute until the WGSL author decides it is needed.

**`ExtendedMaterial` / `MaterialExtension` import paths (verified):**

| Type | Import path |
|---|---|
| `ExtendedMaterial` | `bevy::pbr::ExtendedMaterial` |
| `MaterialExtension` trait | `bevy::pbr::MaterialExtension` |
| `MaterialExtensionKey` | `bevy::pbr::MaterialExtensionKey` |
| WGSL bind group slots | `@group(2) @binding(N)` — standard Bevy material extension convention |

WGSL target: `clients/bevy-ref/assets/shaders/terrain_splat.wgsl`

Tri-planar projection for the `RockCliff` biome: sample three axis-aligned planes (XZ, XY, ZY) weighted by `abs(world_normal)` components. This handles vertical cliffs where UV0 tiling produces visible stretching.

**Step 2 depends on Step 1 being stable.** Do not start Step 2 until Step 1 passes the acceptance screenshot diff.

---

## 2. Post-processing stack — exact Bevy 0.18 component names and module paths

All components below are added to the `Camera3d` entity. The integration point is the `setup_camera` function in `clients/bevy-ref/src/bin/standalone.rs`.

### 2-a. Tonemapping

| Item | Exact identifier |
|---|---|
| Enum | `bevy::core_pipeline::tonemapping::Tonemapping` |
| Crate source | `bevy_core_pipeline-0.18.1/src/tonemapping/mod.rs:119` |
| Recommended variant | `Tonemapping::AcesFitted` — COD-tier cinematic look; no `tonemapping_luts` feature required |
| Alternative | `Tonemapping::AgX` — neutral, requires `bevy/tonemapping_luts` feature |
| Placement | Component on the camera entity |
| Note | `Tonemapping` is NOT re-exported by `bevy::prelude`; import explicitly |

Verified variants: `None`, `Reinhard`, `ReinhardLuminance`, `AcesFitted`, `AgX`, `SomewhatBoringDisplayTransform`, `TonyMcMapface` (default), `BlenderFilmic`.

`AcesFitted` does not require the `tonemapping_luts` feature. `AgX` and `BlenderFilmic` do. Default Bevy features include `tonemapping_luts` — no extra Cargo change needed for `AgX` if using default features.

### 2-b. Bloom

| Item | Exact identifier |
|---|---|
| Struct | `bevy::post_process::bloom::Bloom` |
| Crate source | `bevy_post_process-0.18.1/src/bloom/settings.rs:35` |
| Plugin | `bevy::post_process::bloom::BloomPlugin` — included in `DefaultPlugins`; no manual add needed |
| Required | `Camera { hdr: true }` — Bloom does not render without HDR |
| Starting values | `Bloom { intensity: 0.12, ..Default::default() }` |

The struct is named `Bloom` in Bevy 0.18, not `BloomSettings` (which was the pre-0.15 name). Use `Bloom`.

### 2-c. SSAO / GTAO

| Item | Exact identifier |
|---|---|
| Plugin | `bevy::pbr::ScreenSpaceAmbientOcclusionPlugin` |
| Component struct | `bevy::pbr::ScreenSpaceAmbientOcclusion` |
| Crate source | `bevy_pbr-0.18.1/src/ssao/mod.rs:45, 124` |
| `#[require]` auto-inserts | `DepthPrepass`, `NormalPrepass` |
| Not in `DefaultPlugins` | The agent must add `app.add_plugins(ScreenSpaceAmbientOcclusionPlugin)` |

Verified `#[require]` on the struct:
```
#[require(DepthPrepass, NormalPrepass)]
pub struct ScreenSpaceAmbientOcclusion { ... }
```

Adding `ScreenSpaceAmbientOcclusion` to the camera entity automatically inserts `DepthPrepass` and `NormalPrepass` via Bevy's required-component system. No manual prepass insertion is needed.

Quality levels: `Low`, `Medium`, `High` (default), `Ultra`, `Custom { slice_count, samples_per_slice_side }`.

### 2-d. TAA

| Item | Exact identifier |
|---|---|
| Plugin | `bevy::anti_alias::taa::TemporalAntiAliasPlugin` |
| Component struct | `bevy::anti_alias::taa::TemporalAntiAliasing` |
| Crate source | `bevy_anti_alias-0.18.1/src/taa/mod.rs:49, 123` |
| `#[require]` auto-inserts | `TemporalJitter`, `MipBias`, `DepthPrepass`, `MotionVectorPrepass` |
| Conflict | `Msaa::Off` required — TAA logs warning and skips if MSAA != Off |

Verified `#[require]` on the struct:
```
#[require(TemporalJitter, MipBias, DepthPrepass, MotionVectorPrepass)]
pub struct TemporalAntiAliasing { ... }
```

The agent must add `app.insert_resource(Msaa::Off)` alongside `TemporalAntiAliasPlugin`. `TemporalAntiAliasPlugin` is included inside `AntiAliasPlugin` which is part of `DefaultPlugins` — no separate plugin add needed if `DefaultPlugins` is used.

### 2-e. Cascaded shadow maps

This is a `DirectionalLight` change, not a camera change. Integration point: `setup_atmosphere` in `clients/bevy-ref/src/atmosphere.rs` (the sun light spawn site).

| Item | Exact identifier |
|---|---|
| Field | `DirectionalLight { shadows_enabled: true, .. }` |
| Shadow map size | `app.insert_resource(DirectionalLightShadowMap { size: 4096 })` |
| Cascade config component | `bevy::pbr::CascadeShadowConfigBuilder` |

`CascadeShadowConfigBuilder` target values:
```
CascadeShadowConfigBuilder {
    num_cascades: 4,
    maximum_distance: 500.0,
    ..default()
}.build()
```

This component is inserted in the same spawn bundle as `DirectionalLight`.

### 2-f. Combined camera component checklist

The minimal camera entity component set for the full post stack:

```
Camera { hdr: true, ..default() }
Camera3d::default()
Tonemapping::AcesFitted
Bloom { intensity: 0.12, ..Default::default() }
ScreenSpaceAmbientOcclusion::default()
TemporalAntiAliasing::default()
Msaa::Off
```

Auto-inserted by `#[require]` chains (do not need to be added manually, but explicit insertion is safe and documents intent):
- From `ScreenSpaceAmbientOcclusion`: `DepthPrepass`, `NormalPrepass`
- From `TemporalAntiAliasing`: `TemporalJitter`, `MipBias`, `DepthPrepass`, `MotionVectorPrepass`

`DepthPrepass` is shared — inserting it once satisfies both consumers.

Plugins to add to `App` (beyond `DefaultPlugins`):
- `ScreenSpaceAmbientOcclusionPlugin`

`TemporalAntiAliasPlugin` and `BloomPlugin` are included in `DefaultPlugins`.

---

## 3. bevy_solari GI

### 3-a. Cargo feature

Already wired in `clients/bevy-ref/Cargo.toml`:
```toml
solari = [
    "bevy",
    "bevy/bevy_solari",
]
```
No additional Cargo.toml changes needed.

### 3-b. Plugin addition

In `standalone.rs`:
```
#[cfg(feature = "solari")]
app.add_plugins(bevy::solari::SolariPlugins);
```

`SolariPlugins` is a `PluginGroup` (verified in `bevy_solari-0.18.1/src/lib.rs`) containing:
- `RaytracingScenePlugin` — BLAS construction, resource/lighting binding
- `SolariLightingPlugin` — ReSTIR DI + ReSTIR GI + world-cache indirect + SVGF denoiser

`SolariLightingPlugin::finish()` checks `required_wgpu_features()` and emits a `warn!` + returns early if unsupported — it does not panic. This satisfies the project's "fail clearly, not silently" policy.

### 3-c. Camera component

| Item | Exact identifier |
|---|---|
| Component | `bevy::solari::realtime::SolariLighting` |
| Re-exported in | `bevy::solari::prelude::SolariLighting` |
| Crate source | `bevy_solari-0.18.1/src/realtime/mod.rs:101` |

Verified `#[require]` on the struct:
```
#[require(
    Hdr,
    DeferredPrepass,
    DepthPrepass,
    MotionVectorPrepass,
    DeferredPrepassDoubleBuffer,
    DepthPrepassDoubleBuffer
)]
pub struct SolariLighting { ... }
```

**Important:** `SolariLighting` switches the scene to the deferred rendering pipeline via `DefaultOpaqueRendererMethod::deferred()` (set in `SolariLightingPlugin::build()`). This affects transparency rendering. Test translucent water surface and particles after enabling.

Additional camera requirement (must be set explicitly — not auto-required):
```
CameraMainTextureUsages::default().with(TextureUsages::STORAGE_BINDING)
Msaa::Off
```

### 3-d. Mesh component: `RaytracingMesh3d`

| Item | Exact identifier |
|---|---|
| Component | `bevy::solari::scene::RaytracingMesh3d` |
| Re-exported in | `bevy::solari::prelude::RaytracingMesh3d` |
| Crate source | `bevy_solari-0.18.1/src/scene/types.rs:21` |
| Type signature | `pub struct RaytracingMesh3d(pub Handle<Mesh>)` |

Add to every entity that should participate in RT ray queries (terrain, buildings, trees). Entities without this component are invisible to ray queries — they still render rasterized, but do not cast or receive GI bounces.

Static meshes (terrain, buildings) go in the static BLAS. Dynamic meshes (civilians) update the dynamic TLAS per-frame. `RaytracingScenePlugin` manages BLAS/TLAS construction automatically.

### 3-e. Required wgpu features and backend

Verified in `bevy_solari-0.18.1/src/lib.rs`:
```
WgpuFeatures::EXPERIMENTAL_RAY_QUERY
    | WgpuFeatures::BUFFER_BINDING_ARRAY
    | WgpuFeatures::TEXTURE_BINDING_ARRAY
    | WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
    | WgpuFeatures::PARTIALLY_BOUND_BINDING_ARRAY
```

The RTX 3090 Ti (Ampere, sm_86, DXR 1.1) supports all of these on both DX12 and Vulkan.

To force DX12 (recommended for Windows 11 + DXR path):
```
// Schematic — agent implements in standalone.rs
RenderPlugin {
    render_creation: RenderCreation::Automatic(WgpuSettings {
        backends: Some(Backends::DX12),
        ..Default::default()
    }),
    ..Default::default()
}
```
Or set `WGPU_BACKEND=dx12` in `.env` (preferred per `feedback_secrets_config` convention).

Add a `--no-gi` CLI flag in `standalone.rs` that skips `SolariPlugins`, allowing CI and non-RT hardware to run normally.

### 3-f. TAA + Solari interaction

`SolariLighting` auto-requires `MotionVectorPrepass`. `TemporalAntiAliasing` also auto-requires `MotionVectorPrepass`. They are compatible on the same camera entity — the prepass fires once and feeds both. TAA is required by Solari's denoiser (temporal history). Ensure both components are on the camera entity simultaneously.

---

## 4. DLSS / FSR

### 4-a. DLSS — first-party Bevy 0.18 (not community)

DLSS is built into `bevy_anti_alias-0.18.1` behind the `dlss` feature (verified in `bevy_anti_alias-0.18.1/Cargo.toml`):

```toml
[features]
dlss = ["dep:dlss_wgpu", "dep:uuid", "bevy_render/raw_vulkan_init"]
```

The underlying crate is `dlss_wgpu = "2"` (optional dep). The top-level `bevy` crate forwards it: `bevy/dlss` maps to `bevy_anti_alias/dlss`.

| Item | Detail |
|---|---|
| Bevy feature | `bevy/dlss` |
| Init plugin | `bevy::anti_alias::dlss::DlssInitPlugin` — must be added **before** `DefaultPlugins` |
| Main plugin | `bevy::anti_alias::dlss::DlssPlugin` — auto-added by `DefaultPlugins` when `dlss` feature is on |
| Camera component | `bevy::anti_alias::dlss::Dlss<DlssSuperResolutionFeature>` |
| Camera component (Solari) | `bevy::anti_alias::dlss::Dlss<DlssRayReconstructionFeature>` — use when Solari is active for DLSS Ray Reconstruction |
| Quality enum | `dlss_wgpu::DlssPerfQualityMode` (re-exported as `bevy::anti_alias::dlss::DlssPerfQualityMode`) |
| Runtime capability check | `Option<Res<DlssSuperResolutionSupported>>` — absent if GPU/driver lacks DLSS |
| Auto-required | `TemporalJitter`, `MipBias`, `DepthPrepass`, `MotionVectorPrepass`, `Hdr` |
| Platform | **Vulkan only** (not DX12) — requires `WGPU_BACKEND=vulkan` |
| SDK | NVIDIA Streamline / NGX SDK headers (NDA); see `https://github.com/bevyengine/dlss_wgpu` |
| Conflict | When `Dlss` is active, remove `TemporalAntiAliasing` from the camera — DLSS replaces TAA |

**Critical platform note:** DLSS in `bevy_anti_alias` requires the Vulkan backend. Solari supports both DX12 and Vulkan. When both are active on Windows 11 + RTX 3090 Ti, use Vulkan (`WGPU_BACKEND=vulkan`) to enable both simultaneously. Add a `vulkan` feature or env var to `standalone.rs` to toggle this.

**Cargo addition to `civ-bevy-ref/Cargo.toml`:**
```toml
dlss = ["bevy", "bevy/dlss"]
```

Gate DLSS camera component behind `#[cfg(feature = "dlss")]` in `standalone.rs`.

### 4-b. FSR fallback

Bevy 0.18 does not ship a built-in FSR implementation. `bevy_fsr` community crate is not in the local cargo registry.

Recommended fallback: `bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening` component — post-process sharpening, zero dependencies, works on any GPU. Add to the camera entity as a no-cost quality improvement on non-DLSS hardware.

For FSR 2.x spatial upscaling: add `bevy_fsr` as a deferred `fsr` Cargo feature once the community crate verifies Bevy 0.18 compatibility. Do not block on this.

---

## 5. Effort/impact ranking

| Rank | Task ID | Description | Visual delta | Risk | Agent effort |
|---|---|---|---|---|---|
| 1 | P1 | Post stack: `AcesFitted` + `Bloom` + `ScreenSpaceAmbientOcclusion` + `TemporalAntiAliasing` | Very high — entire scene lifts from "OpenGL demo" to "modern engine" | Very low — all in-tree, no shaders | 5–8 tool calls, ~3 min |
| 2 | M1-a | Terrain tiled PBR: UV0 gen + `BiomeMaterialsPlugin` + six-band mesh split | Very high — eliminates flat color, adds micro-detail | Low — UV math simple, assets on disk | 10–15 tool calls, ~6 min |
| 3 | L1 | Cascaded shadow maps on `DirectionalLight`: 4 cascades, `maximum_distance=500` | High — hard shadow terminator grounds scene | Very low — two field additions | 2–3 tool calls, ~1 min |
| 4 | G1 | `SolariPlugins` + `SolariLighting` + `RaytracingMesh3d` on mesh entities | Very high — indirect bounce, contact shadows | Medium — deferred pipeline switch, DXR backend | 10–15 tool calls, ~8 min |
| 5 | M1-b | `ExtendedMaterial` 6-biome splat WGSL shader | Very high — smooth biome transitions, triplanar cliffs | High — WGSL authoring required | 20–30 tool calls, ~15 min |
| 6 | U1 | DLSS (`bevy/dlss` feature, Vulkan backend) | High — perf + sharpness | High — NDA SDK, Vulkan-only, project UUID | 15–20 tool calls, ~12 min |

**Recommended first 3 highest-ROI lowest-risk changes:**

1. P1 — post stack (zero new deps, zero assets, zero shaders)
2. L1 — cascaded shadows (two field additions in the sun light spawn)
3. M1-a — terrain PBR (assets on disk, step-1 path only)

These three can be executed as one parallel subagent batch (~8 min wall clock, 2–3 subagents), and they produce a visually dramatic result without touching any shared-crate code or authoring WGSL.

---

## 6. Phased WBS + DAG

### Phase table

| Phase | Task ID | Description | Depends On |
|------:|---------|-------------|------------|
| A | A-T1 | Declare `pbr-textures` feature in `Cargo.toml` | — |
| A | A-T2 | Add `ATTRIBUTE_UV_0` to `terrain_mesh()` in `terrain.rs` | — |
| A | A-T3 | `Camera { hdr: true }` + `Tonemapping::AcesFitted` + `Bloom` on camera spawn | — |
| A | A-T4 | `ScreenSpaceAmbientOcclusion` + `TemporalAntiAliasing` + `Msaa::Off` on camera | A-T3 |
| A | A-T5 | `app.add_plugins(ScreenSpaceAmbientOcclusionPlugin)` | A-T3 |
| A | A-T6 | CSM on `DirectionalLight` in `atmosphere.rs`: `shadows_enabled=true`, 4 cascades, `maximum_distance=500` | — |
| B | B-T1 | Wire `BiomeMaterialsPlugin` into `standalone.rs` behind `pbr-textures` | A-T1 |
| B | B-T2 | Split terrain spawn into 6 height-band mesh sections with biome material handles | A-T2, B-T1 |
| B | B-T3 | Building roughness tuning: `perceptual_roughness=0.70`, `reflectance=0.25` | — |
| B | B-T4 | Civilian capsule roughness tuning: `perceptual_roughness=0.55` | — |
| C | C-T1 | `app.add_plugins(SolariPlugins)` behind `#[cfg(feature = "solari")]`; `--no-gi` flag | A-T4 (TAA prereq for denoiser) |
| C | C-T2 | `SolariLighting` + `CameraMainTextureUsages::STORAGE_BINDING` on camera | C-T1 |
| C | C-T3 | `RaytracingMesh3d` on terrain + building + tree entity spawns | C-T1 |
| C | C-T4 | `WGPU_BACKEND=dx12` in `.env`; verify DXR features present | C-T1 |
| D | D-T1 | Add `ATTRIBUTE_BIOME_WEIGHTS` (optional) to terrain mesh | B-T2 |
| D | D-T2 | `TerrainSplatExtension` WGSL: height/slope blending of 6 texture pairs | D-T1 |
| D | D-T3 | Register `ExtendedMaterial<StandardMaterial, TerrainSplatExtension>` | D-T2 |
| D | D-T4 | Tri-planar projection inside WGSL for `RockCliff` biome | D-T3 |
| E | E-T1 | `dlss = ["bevy", "bevy/dlss"]` Cargo feature; `DlssInitPlugin` before `DefaultPlugins` | A-T4 |
| E | E-T2 | `Dlss<DlssSuperResolutionFeature>` camera component; disable TAA when DLSS active | E-T1 |
| E | E-T3 | `Dlss<DlssRayReconstructionFeature>` variant when Solari active; Vulkan backend | E-T2, C-T1 |

### DAG

```
A-T1 ──► B-T1 ──► B-T2 ──► D-T1 ──► D-T2 ──► D-T3 ──► D-T4
A-T2 ──► B-T2
A-T3 ──► A-T4 ──► C-T1 ──► C-T2
A-T3 ──► A-T5          └──► C-T3
                        └──► C-T4
A-T4 ──► E-T1 ──► E-T2 ──► E-T3 ◄── C-T1
A-T6 (independent)
B-T3, B-T4 (independent)
```

**Parallel batch 1** (no predecessors, start together):
`A-T1`, `A-T2`, `A-T3`, `A-T6`, `B-T3`, `B-T4`

**Sequential after batch 1:**
`A-T4` + `A-T5` (after `A-T3`), `B-T1` (after `A-T1`)

**Parallel batch 2** (after batch 1 + sequential):
`A-T4`, `A-T5`, `B-T1`

**Batch 3:**
`B-T2` (after `A-T2` + `B-T1`)

**Batch 4 (optional GI + shader phases):**
`C-T1` (after `A-T4`), `D-T1` (after `B-T2`), `E-T1` (after `A-T4`)

---

## 7. Acceptance criteria

**Phase A + B (post stack + terrain PBR):**
- `cargo run -p civ-bevy-ref --features bevy,pbr-textures --bin civ-standalone` launches without panic.
- Six visually distinct textured biome bands visible on terrain.
- Bloom visible on sun-facing surfaces at low angle.
- SSAO darkens corners between terrain quads and building bases.
- No TAA ghosting during camera pan.
- `cargo check -p civ-bevy-ref --features bevy` (without `pbr-textures`) still builds.
- Frame time < 16.6 ms at 1440p on RTX 3090 Ti.

**Phase C (Solari GI):**
- `cargo run -p civ-bevy-ref --features bevy,solari --bin civ-standalone` launches on DX12.
- Indirect bounce light visible from colored building faces onto adjacent terrain.
- On non-RT driver: app logs loud warning, renders normally without GI.
- Frame time < 11 ms at 1440p with 1 spp + temporal denoising.

---

## 8. Key import paths reference (Bevy 0.18.1 verified)

| Component / Type | Import path | Crate source file |
|---|---|---|
| `Tonemapping` | `bevy::core_pipeline::tonemapping::Tonemapping` | `bevy_core_pipeline-0.18.1/src/tonemapping/mod.rs:119` |
| `Bloom` | `bevy::post_process::bloom::Bloom` | `bevy_post_process-0.18.1/src/bloom/settings.rs:35` |
| `BloomPlugin` | `bevy::post_process::bloom::BloomPlugin` | `bevy_post_process-0.18.1/src/bloom/mod.rs:47` |
| `ScreenSpaceAmbientOcclusion` | `bevy::pbr::ScreenSpaceAmbientOcclusion` | `bevy_pbr-0.18.1/src/ssao/mod.rs:124` |
| `ScreenSpaceAmbientOcclusionPlugin` | `bevy::pbr::ScreenSpaceAmbientOcclusionPlugin` | `bevy_pbr-0.18.1/src/ssao/mod.rs:45` |
| `TemporalAntiAliasing` | `bevy::anti_alias::taa::TemporalAntiAliasing` | `bevy_anti_alias-0.18.1/src/taa/mod.rs:123` |
| `TemporalAntiAliasPlugin` | `bevy::anti_alias::taa::TemporalAntiAliasPlugin` | `bevy_anti_alias-0.18.1/src/taa/mod.rs:49` |
| `DepthPrepass` | `bevy::core_pipeline::prepass::DepthPrepass` | `bevy_core_pipeline-0.18.1/src/prepass/mod.rs:58` |
| `MotionVectorPrepass` | `bevy::core_pipeline::prepass::MotionVectorPrepass` | `bevy_core_pipeline-0.18.1/src/prepass/mod.rs:72` |
| `CascadeShadowConfigBuilder` | `bevy::pbr::CascadeShadowConfigBuilder` | `bevy_pbr-0.18.1` |
| `ExtendedMaterial` | `bevy::pbr::ExtendedMaterial` | `bevy_pbr-0.18.1/src/extended_material.rs:145` |
| `MaterialExtension` | `bevy::pbr::MaterialExtension` (trait) | `bevy_pbr-0.18.1/src/extended_material.rs` |
| `SolariPlugins` | `bevy::solari::SolariPlugins` | `bevy_solari-0.18.1/src/lib.rs:39` |
| `SolariLighting` | `bevy::solari::prelude::SolariLighting` | `bevy_solari-0.18.1/src/realtime/mod.rs:101` |
| `RaytracingMesh3d` | `bevy::solari::prelude::RaytracingMesh3d` | `bevy_solari-0.18.1/src/scene/types.rs:21` |
| `Dlss` | `bevy::anti_alias::dlss::Dlss` | `bevy_anti_alias-0.18.1/src/dlss/mod.rs:217` |
| `DlssInitPlugin` | `bevy::anti_alias::dlss::DlssInitPlugin` | `bevy_anti_alias-0.18.1/src/dlss/mod.rs:60` |
| `DlssSuperResolutionFeature` | `bevy::anti_alias::dlss::DlssSuperResolutionFeature` | `bevy_anti_alias-0.18.1/src/dlss/mod.rs:272` |
| `DlssRayReconstructionFeature` | `bevy::anti_alias::dlss::DlssRayReconstructionFeature` | `bevy_anti_alias-0.18.1/src/dlss/mod.rs:320` |
| `ContrastAdaptiveSharpening` | `bevy::anti_alias::contrast_adaptive_sharpening::ContrastAdaptiveSharpening` | `bevy_anti_alias-0.18.1/src/contrast_adaptive_sharpening/mod.rs:40` |

---

## 9. Cross-project reuse opportunities

Per `CLAUDE.md` Phenotype org reuse protocol — confirm destination with user before extracting:

| Candidate | Current location | Target shared location | Impacted repos |
|---|---|---|---|
| `ATTRIBUTE_BIOME_WEIGHTS` declaration + weight compute | `clients/bevy-ref/src/terrain.rs` (Phase D) | `phenotype-voxel/crates/biome` | Civis, WorldSphereMod3D |
| Tri-planar WGSL helper | `clients/bevy-ref/assets/shaders/terrain_splat.wgsl` | new `phenotype-shaders` crate | Civis, WSM3D |
| Post-processing camera bundle | `clients/bevy-ref/src/bin/standalone.rs` | `phenotype-voxel/crates/bevy-camera` | Any Phenotype Bevy project |
| Height-band biome-from-height mapping | `clients/bevy-ref/src/materials.rs` `from_height_norm` | `phenotype-voxel/crates/biome` | Civis, WSM3D |

Build inline first; extract at second use per `feedback_abstraction_threshold`.

---

## 10. First-batch execution summary

The recommended first three visual upgrades are: (1) enable the full post-processing stack — `Tonemapping::AcesFitted`, `Bloom { intensity: 0.12 }`, `ScreenSpaceAmbientOcclusion` (GTAO), and `TemporalAntiAliasing` on the camera entity using the verified Bevy 0.18 component names above — which transforms the flat plastic rendering into a cinematic look at near-zero risk with no new crate dependencies or shader authoring; (2) switch the `DirectionalLight` in `setup_atmosphere` to cascaded shadow maps with 4 cascades and `maximum_distance=500.0`, giving real shadow gradients across the terrain in two field additions; and (3) add `ATTRIBUTE_UV_0` tiled UV generation to `terrain_mesh()` in `terrain.rs`, declare the `pbr-textures` Cargo feature, wire `BiomeMaterialsPlugin` into `standalone.rs`, and apply the on-disk grass PBR assets via `BiomeMaterials`, replacing the solid `color_for_height` vertex colors with micro-detail surface shading. All three changes require no new external crate dependencies, no WGSL authoring, and no hardware-specific code paths — they are the fastest path from the current "tech-demo visualiser" baseline to a visually credible AAA-adjacent terrain render, and they unlock the prepass infrastructure (depth, motion vectors, normals) that every subsequent feature (Solari GI, DLSS, terrain splat shader) depends on.
