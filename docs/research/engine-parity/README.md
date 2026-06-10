# Engine Parity Research — Hitting AAA Parity in the Civis Bevy/Rust Stack

**Question (user, 2026-05):** State of using Unreal OSS / reverse-engineering for near-100% parity? Same for needed features from Unity, Godot, other engines? C++ interop and C# may be needed sometimes.

**Civis context:** Bevy 0.18, Rust, desktop-primary (DX12 Ultimate, DXR, DLSS), AAA visual bar. Unreal = **showcase client only** ([project_civis_web_primary]). Governing rule: **wrap/borrow > handroll** ([sio2-and-voxel-baselines], [emergence-charter]).

## Documents
- [unreal.md](./unreal.md) — UE license reality, why Lumen/Nanite/Chaos can't be ported, feature→OSS-equivalent table
- [unity.md](./unity.md) — proprietary; concepts only; no interop
- [godot.md](./godot.md) — the one true-OSS (MIT) engine; what to port; godot-bevy bridge status
- [other-oss-engines.md](./other-oss-engines.md) — Veloren, rend3, O3DE/Atom, Flax/Stride, Ambient, Bevy ecosystem
- [interop.md](./interop.md) — C++ interop toolchain (cxx/autocxx/bindgen); C# = avoid

---

## BOTTOM LINE

### Verdict on Unreal reverse-engineering: **recommend AGAINST, wholesale.**
UE5 is **source-available under a proprietary EULA, not OSS.** Copying/translating its source into our MIT/Apache Rust tree is a license violation and license-contaminates the repo; redistribution is restricted to Epic's channels. Beyond license, Lumen/Nanite/Chaos are **deeply C++-coupled to UE's RHI/RDG/GPU-Scene/cook/task-graph** — porting any one ≈ re-implementing a third of UE, for years, to land where clean-room OSS already is. **Read UE source for ideas; never as a code donor. UE stays the showcase client.** Every UE crown jewel already has an in-tree or near-tree Bevy OSS equivalent.

### Engine-by-engine, one line each
- **Unreal:** source-available ≠ OSS; borrow *techniques* via OSS, not code. Showcase client only.
- **Unity:** fully closed; nothing to reverse-engineer; DOTS→Bevy-ECS, Burst→Rust-native are already ours. No interop.
- **Godot:** **MIT, the legal donor** — safe to port; best for navmesh (use `rerecast`), tilemap, particles study. `godot-bevy` bridge exists but tracks Bevy 0.16 (lags our 0.18); optional, not critical.
- **Other:** O3DE Atom (Apache, AAA Forward+ design ref), rend3 (MIT, wgpu patterns), Veloren (GPL — **study only**, planet-scale voxel), Ambient/Stride (compatible, ECS/net ideas).

### Top-10 plug/borrow list — ranked by impact ÷ effort

| # | Plug/borrow | Gives (UE analog) | License | Effort | Why ranked here |
|---|---|---|---|---|---|
| 1 | **`bevy_anti_alias` + `dlss_wgpu` (DLSS / DLSS-RR)** | DLSS upscaling | MIT/Apache + NVIDIA SDK | **Low** | First-party in 0.18, fits our DLSS target — turn on now |
| 2 | **`bevy_solari` (ReSTIR DI/GI + world cache)** | Lumen (RT GI) | MIT/Apache | **Low–Med** | In-tree 0.18; needs DXR (we target it); biggest visual leap per effort |
| 3 | **`meshopt-rs` (meshoptimizer wrap)** | Nanite cook (meshlet build/simplify) | MIT | **Low** | Drop-in FFI wrapper; foundation for virtual geometry |
| 4 | **`rerecast` / `bevy_rerecast` (navmesh)** | UE/Unity/Godot Recast navmesh | clean-room Rust | **Low–Med** | Replaces hand-rolled pathfinding; emergence agents need it |
| 5 | **`bevy_hanabi` (GPU particles)** | Niagara VFX | MIT/Apache | **Low–Med** | Mature ecosystem; visual polish |
| 6 | **Bevy in-tree virtual geometry / `MeshletPlugin`** | Nanite (virtualized geometry) | MIT/Apache | **Med–High** | ~60–70% of Nanite; experimental, DAG-quality WIP — adopt + track |
| 7 | **`Avian` or `bevy_rapier` (rigid bodies)** | Chaos rigid debris | MIT/Apache | **Med** | Destruction = our voxel CA + rigid debris; no Chaos port |
| 8 | **`Scthe/nanite-webgpu` (reference design)** | Nanite SW-raster + impostors + cull | MIT | **Med** (study) | Borrow DAG/cluster/impostor ideas to harden #6 |
| 9 | **O3DE Atom (design reference)** | Forward+ data-driven pipeline | Apache | **Low** (read) | AAA pipeline design source; no runtime dep |
| 10 | **Jolt-C / NanoVDB (conditional C++ wraps)** | Large-scene physics / sparse volumes | MIT / MPL | **Med–High** | Only if profiling proves Rapier/our-SVO insufficient |

(Veloren = study-only due to GPL; godot-bevy = optional secondary editor route; FSR/XeSS = wrap later for non-NVIDIA upscaling.)

### Interop recommendation
**Stay pure-Rust by default.** Use C++ interop *only* to wrap a best-in-class native C++ lib with no Rust equal — and prefer the **existing wrapper crate** (`meshopt-rs`, `dlss_wgpu`, `rerecast`, `physx-rs`, `jolt-rust`) over hand-rolled FFI. New bindings: **`cxx`** (first choice) → **`autocxx`** (large header sets) → **`bindgen`** (C / C-shaped APIs, e.g. JoltPhysics-C, NanoVDB). **C# interop: avoid** — never host the CLR for one library; the underlying value is always a native C++ lib you can wrap directly. Check each lib's license (all candidates MIT/Apache/BSD/zlib/MPL-compatible; NVIDIA DLSS SDK has its own redistribution terms).

---

## Sources
- [Bevy 0.18](https://bevy.org/news/bevy-0-18/) · [bevy_solari 0.18](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18/) · [Bevy 0.17 (DLSS/dlss_wgpu)](https://bevy.org/news/bevy-0-17/)
- [Virtual Geometry in Bevy 0.16](https://jms55.github.io/posts/2025-03-27-virtual-geometry-bevy-0-16/) · [Scthe/nanite-webgpu](https://github.com/Scthe/nanite-webgpu) · [meshopt-rs](https://github.com/gwihlidal/meshopt-rs)
- [Unreal Engine EULA](https://www.unrealengine.com/eula/unreal) · [UE licensing](https://www.unrealengine.com/license)
- [rerecast](https://github.com/janhohenheim/rerecast) · [godot-bevy](https://github.com/bytemeadow/godot-bevy) · [O3DE (Apache)](https://github.com/o3de/o3de/) · [rend3](https://github.com/BVE-Reborn/rend3)
- [autocxx](https://google.github.io/autocxx/) · [cxx.rs](https://cxx.rs/context.html)
