# Civis Bevy AAA-Quality Roadmap

Date: 2026-05-28
Target hardware: RTX 3090 Ti (Ampere, 24 GB), DX12 Ultimate (mesh shaders, DXR 1.1, VRS, sampler feedback), Windows 11.
Target visual bar: COD / Rust-tier — PBR materials, real GI, animated humanoids, dense vegetation, modern post stack, DLSS.

Baseline today (`clients/bevy-ref`, Bevy 0.18, `bevy_egui` 0.39, `wgpu` 27, `bevy_solari` 0.18 feature-flagged):
- Terrain mesh + 4 hard-coded building cubes + capsule civilians.
- Solid-color `StandardMaterial` (no textures, no normals).
- Sun/moon directional lights + star skybox; no atmospheric scattering, no SSAO/bloom/TAA.
- No animation, no LOD, no streaming, no GI yet (feature compiled but not on), no upscaler.

This roadmap is the path from "tech-demo visualizer" to AAA-tier desktop renderer. It is structured as a phased WBS with a DAG. No code, only decisions, work packages, and acceptance criteria.

---

## 1. Impact / Effort Matrix

Effort uses the agent-led scale from `CLAUDE.md` (tool calls / parallel subagents / wall-clock).

| ID | Work Package | Visual Impact | Effort | ROI |
|----|--------------|---------------|--------|-----|
| M1 | PBR material upgrade (albedo/normal/roughness/AO on terrain + buildings) | Very High | Small (8–15 tool calls, ~5 min) | Top |
| P1 | Post stack: TAA + bloom + tonemapping (ACES) + SSAO | Very High | Small (6–10 tool calls, ~3 min) | Top |
| A1 | bevy_atmosphere sky + scattering | High | Small (4–8 tool calls, ~3 min) | Top |
| C1 | Replace civilian capsules with low-poly humanoid GLTF + idle/walk anim | Very High | Medium (15–25 tool calls or 3 subagents, ~10 min) | High |
| V1 | Vegetation pass: instanced trees + grass, wind shader | Very High | Medium (15–25 tool calls or 3 subagents, ~10 min) | High |
| G1 | Enable `bevy_solari` GI (RT direct + indirect) on 3090 Ti | Very High | Medium (8–15 tool calls, ~5 min, debugging dominant) | High |
| U1 | DLSS via `dlss_wgpu` / FSR 3.1 fallback (`bevy_fsr`) | High (perf + sharpness) | Medium (15–25 tool calls, ~10 min) | High |
| S1 | DirectStorage / async chunk streaming for world tiles | Medium (enables scale) | Large (3–5 subagents, ~20 min) | Medium |
| B1 | Building variety: modular kit + Quixel megascans free tier | High | Medium (~10 min) | High |
| L1 | Cascaded shadow maps + contact shadows | Medium | Small (~3 min) | High |

Headline ranking: do M1 + P1 + A1 + L1 first — biggest perceived jump for least effort. Then C1, V1, B1 (content density). Then G1 + U1 (the "wow" tech layer). S1 last, only when world size demands it.

---

## 2. Phased WBS (DAG)

### Phase 0 — Asset pipeline foundations
Predecessor: none.

- P0.1 Set up `assets/` subtree convention: `materials/{terrain,architecture,vegetation,characters}/<name>/{albedo,normal,arm,ao,height}.ktx2`.
  - ARM = packed (AO, Roughness, Metallic) — Bevy `StandardMaterial.metallic_roughness_texture` + `occlusion_texture`.
- P0.2 Add `ktx2` + `zstd` features to the Bevy dependency so we ship compressed textures (BC7 on desktop). Bevy 0.18 supports KTX2 + zstd out of the box; verify feature flags in `Cargo.toml`.
- P0.3 Decide texture authoring source-of-truth: PolyHaven CC0 (4k PBR) is primary, ambientCG CC0 secondary, Quixel free-tier (now free for everyone after Epic's 2025 policy change, CC-ish license but with attribution) tertiary for buildings.
- P0.4 Write a one-shot importer script (Rust binary in `tools/`) that takes a PolyHaven zip, transcodes to KTX2 BC7, and emits a manifest TOML.
- Acceptance: one terrain material round-trips PolyHaven → KTX2 → Bevy preview without manual touch-up.

### Phase 1 — Material upgrade (M1, L1)
Predecessor: P0.

- M1.1 Replace terrain solid color with PolyHaven `rocky_terrain_02` (or `forrest_ground_01`) at 4k, triplanar-blended via a custom material or `bevy_terrain` crate. Standalone Bevy can also just UV-tile if terrain is flat-ish — start there.
- M1.2 Building cubes: swap to `concrete_wall_006` + `brick_wall_006` from PolyHaven; per-faction tint via material `base_color` multiplier.
- M1.3 Add `NormalMap` + `MetallicRoughness` + `AO` to all materials — verify with a "material sphere" debug scene.
- L1.1 Enable cascaded shadow maps on the sun light (Bevy `DirectionalLight` already supports CSM in 0.18; set `shadow_depth_bias`, `num_cascades=4`, `maximum_distance=500.0`).
- L1.2 Add contact shadows (screen-space) via `ScreenSpaceAmbientOcclusion`'s sibling — Bevy 0.18 exposes `ScreenSpaceReflectionsBundle` and an SSAO bundle; contact shadows are part of the standard PBR pipeline once SSAO is on.
- Acceptance: standalone scene shows surface micro-detail under sun, AO in crevices, sharp shadows.

### Phase 2 — Post-processing stack (P1, A1)
Predecessor: Phase 1 (so the materials actually have something to show).

- P1.1 Enable `bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted` on the main camera. Bevy 0.18 ships AGX, Reinhard, ACES, Blender Filmic — pick ACES Fitted for COD-tier look or AGX for neutral.
- P1.2 Add `bevy::core_pipeline::bloom::BloomSettings` (intensity ~0.15, default config).
- P1.3 Add `bevy::pbr::ScreenSpaceAmbientOcclusionBundle` (SSAO-GTAO in Bevy 0.18) — gated behind a quality setting.
- P1.4 Add `bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle` (TAA). Required for Solari and any RT effect (denoiser leans on temporal reprojection).
- P1.5 Add `bevy_hanabi` 0.18 for particle FX (sparks, dust, weather). crates.io: `bevy_hanabi` tracks Bevy version; verify `bevy_hanabi = "0.18"`.
- A1.1 Add `bevy_atmosphere` (crates.io: latest 0.18-compatible version — check `bevy_atmosphere = "0.13"` or newer at audit time; this crate has lagged historically, fallback is the built-in `AtmosphereSettings` planned for Bevy 0.19). Replace the manual sun/moon sky cube with Nishita scattering.
- A1.2 Couple atmosphere sun direction to the existing day/night cycle resource.
- Acceptance: real-time sunset shows orange scattering, blue hour, stars only at low sun elevation; bloom blooms; AO darkens corners; TAA stable on movement.

### Phase 3 — Characters (C1)
Predecessor: Phase 1 (materials), Phase 2 (post stack, so characters look right).

- C1.1 Source low-poly humanoid GLTF — recommended free sources, in order:
  1. **Mixamo** (Adobe, free, requires login; rigged + 2000+ animations). License permits game use, no attribution required.
  2. **Quaternius** (CC0, very low-poly stylized humans — fits a "civ sandbox" aesthetic perfectly, free pack `Ultimate Modular Characters`).
  3. **Kenney.nl** (CC0, blocky characters).
  4. **Sketchfab** filtered by CC0/CC-BY.
- C1.2 Pick Quaternius for the civilian baseline (uniform style, ~500 tris each, single material → instanceable). Reserve Mixamo for hero units that need higher fidelity.
- C1.3 GLTF load via Bevy's built-in loader (`gltf` feature is on by default with `bevy`'s default features).
- C1.4 Animation: use Bevy 0.18's `AnimationGraph` + `AnimationPlayer`. Map sim state (idle / walking / fighting) to animation nodes via `AnimationTransitions`.
- C1.5 Skinning: GPU skinning is automatic in Bevy 0.18 for skinned meshes; verify with `RenderDoc` that the skin compute pass fires.
- C1.6 Faction tint via material override on a `_color_mask` texture channel (or per-instance via `MaterialExtension`).
- C1.7 LOD: at >100 m camera distance, swap to a static imposter quad (use `bevy_mod_imposters` if available; otherwise a billboarded `Sprite3d` from `bevy_sprite3d`).
- Acceptance: 1000 civilians animate walk cycle, four faction tints visible, frame time <8 ms on 3090 Ti.

### Phase 4 — Vegetation + buildings (V1, B1)
Predecessor: Phase 1 (materials pipeline), Phase 3 (instancing patterns proven).

- V1.1 Trees: use **Quaternius `Ultimate Nature Pack`** (CC0, low-poly, ~50 tree variants). For higher-fidelity hero trees, use **PolyHaven CC0 tree scans**.
- V1.2 SpeedTree alternative for Bevy: there is no first-party SpeedTree runtime for Rust. Options:
  - `bevy_foliage` (community crate, 0.18-tracking) — wind shader + GPU instancing.
  - Hand-rolled `InstancedMaterial` via Bevy's built-in `MeshInstance` (Bevy 0.18 has automatic instancing for shared meshes with `InheritedVisibility`).
  - Use `MaterialExtension` to add a vertex-shader wind ripple (sin(time + worldpos.xz) on the upper LOD vertices, masked by vertex color alpha).
- V1.3 Grass: use `bevy_grass` (community crate) or a custom compute-shader-driven instanced quad field; cap at 50 k blades around the camera with a ring-buffer streaming approach.
- V1.4 Cull beyond 200 m, use imposter cards beyond 100 m.
- B1.1 Modular building kit: Kenney `City Kit` (CC0) for blocky civ buildings, or Quaternius `Ultimate Modular Buildings` (CC0). Per-faction palette swap.
- B1.2 Replace the 4 hardcoded cubes with a procedural placement that maps sim `Building.kind` → kit piece.
- Acceptance: 200 trees, 50 buildings, grass within 30 m radius, all <16 ms frame.

### Phase 5 — Ray-traced GI (G1)
Predecessor: Phase 2 (TAA is mandatory for the Solari denoiser), Phase 3 (skinned meshes need to be in the BLAS).

- G1.1 Enable the `solari` feature in the `civ-bevy-ref` build (already wired per `docs/research/bevy-gi-status.md`).
- G1.2 Add `bevy::solari::SolariPlugins` after `DefaultPlugins`.
- G1.3 Verify wgpu backend is DX12 (`WGPU_BACKEND=dx12` env or `RenderPlugin { wgpu_settings: WgpuSettings { backends: Some(Backends::DX12), .. }, .. }`). Solari requires RT-capable backend (DX12 with DXR or Vulkan with `VK_KHR_ray_tracing_pipeline`). 3090 Ti supports both.
- G1.4 Mark dynamic meshes (skinned civilians) for inclusion in the dynamic TLAS. Static meshes (terrain, buildings, trees) go in the static BLAS — set the `RaytracingMesh3d` component accordingly. (Bevy 0.18 docs name varies; check `bevy_solari::scene::RaytracedMesh` or similar marker component.)
- G1.5 Tune denoiser settings — Solari ships a SVGF-like denoiser; expose `SolariSettings { samples_per_pixel: 1, denoiser_strength: ..}` in `civ-watch`.
- G1.6 Fallback path: if Solari fails to init (older driver, non-RT hardware), log loud failure and skip plugin — per project policy in `CLAUDE.md` ("fail clearly, not silently"). Add a `--no-gi` flag.
- G1.7 Backup if Solari is unstable in production: investigate `bevy_radiance_cascades` (community), DDGI probe approaches, or stay on Solari's path tracer for offline reference shots. Documented in `docs/research/bevy-gi-status.md`.
- Acceptance: indirect bounce visible on indoor scenes (open a building roof, see colored bleed); frame time <11 ms at 1440p with TAA + Solari + 1 spp.

### Phase 6 — Upscaling (U1)
Predecessor: Phase 2 (TAA pipeline + motion vectors are reused by DLSS/FSR), ideally Phase 5 (RT effects benefit most from upscaling).

- U1.1 DLSS in Rust — current state of the ecosystem (as of 2026-05):
  - **`dlss_wgpu`** (community, GitHub `ForestKitten/dlss-wgpu` and forks): Rust wrapper around Streamline / NGX SDK. Status: experimental, requires NVIDIA NDA-distributed SDK headers. Workable on Windows/DX12. License: depends on NVIDIA Streamline (MIT-ish but with NDA bits).
  - **`bevy_dlss`** (community plugin, tracks Bevy): wires `dlss_wgpu` into Bevy's upscaling pipeline. Verify crate exists and Bevy version match before committing.
  - Realistic path: vendor `dlss_wgpu` as a git dep, hide behind `dlss` feature flag, fall back to FSR by default.
- U1.2 FSR fallback — always-available, MIT-licensed:
  - **`bevy_fsr`** or built-in: Bevy 0.18 has a built-in `Upscaling` enum (`Bilinear`, possibly `Fsr1`); for FSR 3.1 / FSR 2.x you need a wrapper. **`fsr-rs`** wraps AMD's FidelityFX-SDK shaders (HLSL → naga-translated).
  - Recommended: ship FSR 2.2 spatial+temporal via the `bevy_fsr` community crate (or hand-port the public FSR 2 shaders — ~600 LOC HLSL → WGSL).
- U1.3 Render at 1440p internal, upscale to 4k native. Couple to TAA (DLSS replaces TAA; FSR 2 also replaces TAA — disable the Bevy TAA when upscaler is active).
- U1.4 Expose quality presets: `Performance` (1080→4k), `Balanced` (1253→4k), `Quality` (1440→4k), `DLAA` (4k native, AI AA only).
- Acceptance: 4k output at >100 fps with full effects on 3090 Ti; fallback to FSR works on any GPU including non-RT.

### Phase 7 — Streaming / scale (S1)
Predecessor: everything else (only needed when world > what fits in VRAM).

- S1.1 DirectStorage in Rust: **`direct-storage-rs`** (community wrapper around the DirectStorage 1.2 COM API) — Windows-only, DX12-only. Status as of 2026-05: thin bindings exist, no Bevy integration; you'd write a custom `AssetReader` that pulls KTX2 chunks straight from NVMe to VRAM via GPU decompression.
- S1.2 Realistic alternative: async chunk loading via Bevy's built-in `AssetServer` + `tokio` background tasks reading from `MinIO` (already in stack per `project_civis_infra_stack`). Good enough until world > 16 km².
- S1.3 World-space chunking: 256 m tiles, 5×5 around camera kept in memory (~10 GB textures budget on 3090 Ti's 24 GB).
- S1.4 GPU decompression: BC7 is hardware-decoded; for further compression, use GDeflate via DirectStorage GPU decompression path.
- Acceptance: 4 km × 4 km world streams without stalls; chunk transitions invisible.

---

## 3. DAG (dependency edges)

```
P0 ── Phase 0 asset pipeline
 └──► M1 + L1 (Phase 1: materials, shadows)
       └──► P1 + A1 (Phase 2: post, sky)
             ├──► C1 (Phase 3: characters)
             │     └──► V1 + B1 (Phase 4: vegetation + buildings)
             │           └──► G1 (Phase 5: Solari GI; needs C1 skinned meshes in TLAS, P1 TAA)
             │                 └──► U1 (Phase 6: DLSS/FSR)
             └──► S1 (Phase 7: streaming, independent — can start in parallel after Phase 1)
```

Parallelizable batches:
- After Phase 0: Phase 1 alone.
- After Phase 1: Phase 2 + start Phase 3 character sourcing in parallel.
- After Phase 2: Phase 3 implementation + Phase 4 asset sourcing + Phase 7 design.
- After Phase 4: Phase 5 GI + Phase 6 upscaler.

---

## 4. Asset sources reference

| Asset class | Primary (CC0) | Secondary | Tertiary |
|---|---|---|---|
| PBR materials | PolyHaven (polyhaven.com) | ambientCG (ambientcg.com) | Quixel Megascans free tier (now fully free post-2025) |
| Humanoid characters | Quaternius `Ultimate Modular Characters` | Kenney.nl | Mixamo (rigged + animated, Adobe free login) |
| Trees / foliage | Quaternius `Ultimate Nature Pack` | PolyHaven CC0 tree scans | Sketchfab CC0 filter |
| Buildings | Kenney `City Kit` / Quaternius modular buildings | PolyHaven HDRIs (for env reflections) | Quixel Megascans architecture |
| HDRIs (env light) | PolyHaven (CC0, 8k EXR available) | ambientCG | — |
| Animations | Mixamo (Adobe free) | Bevy `bevy_animation_graph` examples | — |
| Particle FX assets | bevy_hanabi examples (MIT) | Kenney particle packs | — |

All sources above are free for commercial use; PolyHaven, ambientCG, Quaternius, Kenney = pure CC0 (no attribution required). Mixamo = Adobe ToS allows game use without attribution.

---

## 5. Crate / version pinboard

These should be added to `clients/bevy-ref/Cargo.toml` as new features in dependency order. Verify each version against crates.io at implementation time (versions move fast).

| Purpose | Crate | Likely version (verify) | Feature flag |
|---|---|---|---|
| Atmospheric sky | `bevy_atmosphere` | 0.13.x (or follow-on) | `atmosphere` |
| Particle FX | `bevy_hanabi` | 0.18.x | `fx` |
| Imposters / billboards | `bevy_sprite3d` or `bevy_mod_imposters` | check | `imposters` |
| Grass field | `bevy_grass` (community) | check | `grass` |
| Foliage instancing | `bevy_foliage` (community) | check | `foliage` |
| Terrain | `bevy_terrain` (community) | 0.18.x | `terrain` |
| DLSS | `dlss_wgpu` + `bevy_dlss` (git deps, NDA SDK) | git rev | `dlss` |
| FSR fallback | `bevy_fsr` (community) or hand-ported FSR2 WGSL | check | `fsr` |
| DirectStorage | `direct-storage-rs` (community) | check | `directstorage` |
| Ray-traced GI | `bevy_solari` (in-tree as Bevy feature) | matches Bevy 0.18 | `solari` (already wired) |
| Asset loader for KTX2 | built-in to `bevy` | enable `ktx2` + `zstd` features | always on |

Always-on crates (no feature flag): `bevy` (with `ktx2` + `zstd` + `bevy_solari` available), `bevy_egui` (already present).

---

## 6. Quality gates per phase

Each phase must hit these before merging to `main`:

- Frame time < 16.6 ms (60 fps) at 1440p on the 3090 Ti reference machine (the user's desktop).
- No new lint warnings; clippy clean.
- A side-by-side screenshot diff in `docs/reports/` showing before/after of the standalone scene.
- The `civ-standalone` binary still launches and ticks the simulation.
- Determinism: rendering changes must not perturb the simulation tick stream (verified by the existing determinism harness).

---

## 7. Out of scope (explicitly deferred)

- Hair / cloth simulation (premature for a civ-sandbox).
- Path tracing as the runtime renderer (Solari is hybrid — RT for GI only, raster for primary visibility). The Bevy `PathtracingPlugin` stays an offline reference tool.
- Nanite-style virtual geometry (no Bevy equivalent exists; mesh shaders via wgpu 27 are the future plumbing).
- Lumen-style software fallback for non-RT GPUs — Solari already requires RT; FSR fallback covers the non-RT user without GI.
- Multi-GPU. 3090 Ti is the only target.

---

## 8. Cross-project reuse opportunities

Per the Phenotype org reuse protocol:

- The KTX2 / PolyHaven importer (P0.4) should live in `phenotype-voxel/tools/` or a new `phenotype-assets` crate so WorldSphereMod3D, Civis, and DINOForge share it.
- The DLSS/FSR upscaler wrapper (U1) should be its own crate `phenotype-upscale` — useful for any Bevy-based Phenotype project.
- The `bevy_atmosphere` + day/night coupling (A1) overlaps with WSM3D's existing sky cycle; consolidate into `phenotype-sky` if the same pattern emerges there.
- The Quaternius character loader + LOD imposter system (C1) is reusable across any civ/RTS-class Phenotype game.

Confirm destinations with the user before extraction; for now build inline in `clients/bevy-ref` and flag the extraction candidates.

---

## 9. Recommended first batch (next session)

Do these in parallel as one subagent batch (~10 min wall clock):

1. M1.1 + M1.2 + M1.3 — drop in 2 PolyHaven materials, wire normal+ARM+AO on terrain + cubes.
2. P1.1 + P1.2 + P1.3 + P1.4 — flip on ACES tonemapping + bloom + SSAO + TAA (one component per camera).
3. L1.1 — set CSM cascades on the sun.
4. A1.1 — add `bevy_atmosphere` Nishita sky.

Single screenshot diff after the batch should already look 3-4x more "AAA" than today's flat-shaded baseline. Everything else (characters, vegetation, GI, DLSS) flows from there.
