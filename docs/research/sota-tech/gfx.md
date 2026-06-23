# SOTA + Experimental Graphics → Civis (Bevy 0.18 voxel world)

**Scope:** rendering techniques mapped to Civis's voxel world + AAA visual bar. **This deliberately does NOT repeat the [engine-parity README](../engine-parity/README.md) plug-list** (DLSS/`dlss_wgpu`, `bevy_solari`, `MeshletPlugin`/virtual geometry, `meshopt-rs`, `bevy_hanabi`, `bevy_anti_alias` are already ranked there — see that table). This doc covers the *techniques* and *experimental bets* around those plugs, with voxel-specific fit. Companion: [material-physics.md](./material-physics.md) (what we're rendering).

## Bevy 0.18 baseline (what's in-tree now)
- **`bevy_solari`** — real-time raytracing, now at **0.19 (April 2026)**: ReSTIR DI (direct) + ReSTIR GI (2nd bounce) + world-space irradiance cache (further bounces) + a **specular GI pass** (0–3 bounce pathtracing for reflections/mirrors). Requires DXR (we target it). This is our Lumen analog — *the* GI answer. Recent fixes: light-tile energy-loss bug, ReSTIR resampling bias, more reactive world cache on large scenes.
- **GPU-driven rendering** (since 0.16) + **procedural atmospheric scattering** (Earth-like sky, any time of day, cheap) — both in-tree, turn on now.
- **DLSS / DLSS-RR**, **virtual geometry (meshlets)**, **GPU particles** — see engine-parity table.

---

## 1. Global Illumination — the decision `[adopt-now: Solari; VXGI avoid]`
- **ReSTIR-GI via `bevy_solari`** `[adopt-now]` — already the answer (in-tree, DXR, biggest visual leap per effort per engine-parity #2). Use it.
- **DDGI / surfels** `[avoid for us]` — probe-based (DDGI) and surfel GI are strong on hardware without RT, but **Solari's ReSTIR + world-cache has effectively superseded them in the Bevy roadmap** (no active DDGI/surfel work in-tree). Skip; would be parallel effort against the maintained path.
- **VXGI / Voxel Cone Tracing** `[experimental, tempting but avoid as primary]` — *superficially* perfect for us (we already have a voxel world → the voxelization step is free, unlike for triangle engines). `bevy-vxgi` exists as a community plugin. BUT: VCT is the *previous generation* of real-time GI (light leaking, blurry specular, cone-trace cost); Solari/ReSTIR is strictly better quality. **Only consider VXGI as a fallback GI for non-DXR hardware** (since our voxel data makes voxelization cheap) or as a cheap far-field/LOD bounce. Tag experimental; do not invest as primary GI.

## 2. Reflections — SSR + RT `[adopt-now]`
- **RT reflections** now come *for free-ish* via Solari's **specular GI pass** (0–3 bounce) at 0.19 — mirrors/water/metal handled by the same system. Known rough edges at 0.19 (mirror artifacts, non-metal BRDF correctness, world-cache light leaks being fixed). Track upstream.
- **SSR** (screen-space reflections) `[adopt-now]` — Bevy has SSR; use as the cheap default + fallback where Solari specular is off/too costly. Standard hybrid: SSR near-field, RT specular for off-screen.

## 3. Volumetric clouds + atmosphere `[adopt-next]`
- **Atmosphere** `[adopt-now]` — Bevy's in-tree procedural atmospheric scattering (0.16+) gives physically-based sky/sunset/day-night at minimal cost. Turn on now; ties to charter `crates/planet` insolation/day-night.
- **Volumetric clouds** `[adopt-next]` — not in-tree; the SOTA recipe is **raymarched dual-layer procedural-noise (Worley+Perlin) volumes with ray-cast lighting** (the Horizon/Decima/Nubis lineage; many open wgpu/GLSL references). Implement as a Bevy post/volumetric pass. ML-driven cloud shaders (UE research, 2025) are an experimental flourish — not needed. Clouds should read `crates/planet` weather/humidity → emergent weather visuals. Tag adopt-next.
- **Volumetric fog / god-rays** `[adopt-now]` — Bevy has volumetric fog; cheap atmosphere win, pairs with the above.

## 4. Virtual texturing `[adopt-next / experimental]`
For 20mi×20mi unique-textured terrain, **virtual texturing** (sparse/streamed mega-texture, only resident tiles in VRAM) is the classic answer (id Tech / Trials). Not in-tree in Bevy; would be a significant custom wgpu effort (sparse residency, feedback pass, tile streaming). For a *voxel* world we may sidestep much of it: voxel materials are mostly **palette/triplanar-shaded**, not uv-unwrapped textures, so per-voxel material IDs + a small texture-array atlas covers most needs without full VT. **Recommend triplanar + material-array first** `[adopt-now]`, full virtual texturing only if unique high-detail decals/terrain-painting demand it `[experimental]`.

## 5. GPU-driven rendering `[adopt-now]`
In-tree since 0.16 (GPU culling, indirect draw, batching). For a voxel world the bigger win is **rendering directly from GPU voxel/CA buffers** (see material-physics Tier-1 GPU CA) — mesh on GPU (greedy meshing / surface-nets in compute) + GPU-driven indirect draw, avoiding CPU readback. This is the scale path for the voxel substrate; combine with meshlet virtual geometry for distant LOD.

## 6. Upscalers — DLSS / FSR / XeSS `[adopt-now: DLSS; next: FSR/XeSS]`
- **DLSS / DLSS-RR via `dlss_wgpu`** `[adopt-now]` — first-party in Bevy 0.17+, matches our NVIDIA/DLSS target, pairs with Solari (DLSS-RR denoises RT). Per engine-parity #1.
- **FSR 3 / XeSS** `[adopt-next]` — for AMD/Intel/non-RTX users. FSR is open-source (GPUOpen); XeSS SDK from Intel. Wrap later for non-NVIDIA parity (engine-parity notes "wrap later"). Frame-gen (DLSS-FG/FSR-FG) experimental.
- **TAA / HDR** `[adopt-now]` — Bevy has TAA (often required as the denoise/accumulation base for RT + a meshlet-AA need) and HDR/tonemapping. NB charter memory: keep default tonemapping features (the `default-features=false` strips tonemapping LUTs → black PBR pitfall, per project memory). Use the **AgX/TonyMcMapface** tonemappers for the AAA filmic look.

## 7. Gaussian splatting & neural rendering `[experimental bet]`
- **3D Gaussian Splatting (3DGS)** `[experimental]` — SIGGRAPH 2023 best paper; photoreal capture rendered at 100+ FPS *without a neural net at render time*. Game-engine support is arriving (Unity/UE plugins; WebGPU renderers like "Visionary" 2025). **Civis fit is narrow but real:** (a) photoreal *skyboxes / distant matte backdrops* / set-dressing captured as splats; (b) hero props/landmarks. It does **not** fit our *simulated, mutable* voxel world (splats are static captures, can't be CA-simulated or destroyed). wgpu 3DGS renderers exist to borrow. Tag experimental bet — a visual flourish for static backdrops, never the world substrate.
- **Neural rendering / world-models** `[experimental, watch]` — per-frame ONNX neural post (super-res, neural materials, NeRF-in-engine) is research-stage for interactive engines. Watch; not actionable for Civis now. DLSS *is* the one shipping neural technique we use.

---

## Verdict (gfx)
**Adopt-now:** `bevy_solari` ReSTIR GI+specular (GI + RT reflections in one), in-tree procedural atmosphere + volumetric fog, GPU-driven rendering (+ render-from-GPU-voxel-buffer), DLSS/DLSS-RR, TAA + AgX tonemapping (keep default tonemapping LUTs), triplanar+material-array voxel shading, SSR as cheap reflection fallback.
**Adopt-next:** raymarched volumetric clouds (planet-weather-driven), FSR/XeSS for non-NVIDIA, full virtual texturing only if decal/terrain-paint detail demands it.
**Experimental bets:** VXGI *only* as non-DXR GI fallback (our voxels make voxelization cheap) — not primary; 3D Gaussian Splatting for static photoreal backdrops/landmarks (never the mutable world); neural rendering — watch.
**Avoid:** DDGI/surfels (superseded by Solari in-tree); VXGI as primary GI.

## Sources
- [Realtime Raytracing in Bevy 0.19 (Solari) — jms55, Apr 2026](https://jms55.github.io/posts/2026-04-12-solari-bevy-0-19/) · [Solari 0.18](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18/) · [Solari 0.17](https://jms55.github.io/posts/2025-09-20-solari-bevy-0-17/)
- [Bevy 0.16: GPU-driven rendering + procedural atmospheric scattering](https://alternativeto.net/news/2025/4/bevy-0-16-released-with-gpu-driven-rendering-procedural-atmospheric-scattering-and-more/)
- [bevy-vxgi (voxel cone traced GI)](https://github.com/Dimev/bevy-vxgi)
- [Real-time GPU volumetric clouds (raymarched, dual-noise)](https://github.com/Depersonalizc/volumetric-clouds) · [Efficient real-time GPU cloud rendering (MDPI)](https://www.mdpi.com/2073-8994/10/4/125)
- [3D Gaussian Splatting in game dev (KIRI)](https://www.kiriengine.app/blog/3DGaussianSplatting_GameDevelopment) · [Visionary: WebGPU Gaussian Splatting platform (arXiv 2512.08478)](https://arxiv.org/html/2512.08478v1)
