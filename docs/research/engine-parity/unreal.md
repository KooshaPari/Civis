# Unreal Engine — Parity Path for Civis (Bevy 0.18 / Rust desktop)

**Scope:** Civis renders/sims in Bevy 0.18 (DX12 Ultimate, DXR, DLSS). UE is a **showcase client only** ([project_civis_web_primary]). This doc answers: can we reuse UE source/OSS, why reverse-engineering Lumen/Nanite/Chaos into Rust is impractical, and the realistic technique-borrowing path.

---

## 1a. UE5 license reality — "source-available", NOT OSS

UE5 is **source-available under a proprietary EULA**, not open source. Concretely, per the Unreal Engine EULA and licensing pages:

- Engine source is on the private `EpicGames/UnrealEngine` GitHub (account linking required), licensed under the **Unreal Engine EULA**, not MIT/Apache/GPL.
- You may distribute Engine Code (incl. your modifications) **only to third parties separately licensed by Epic for the same Engine version**. Public distribution of Engine Tools must go through an Epic-operated marketplace (Fab) or a fork of Epic's GitHub network. (Unreal Engine EULA.)
- Commercial royalty: **5% of lifetime gross revenue above $1M USD** directly attributable to the UE product. (Unreal Engine licensing page.)

**What this means for Civis:**
- ❌ **Cannot** copy/translate UE C++ source (Nanite, Lumen, Chaos, renderer) into our MIT/Apache Rust crates. That is a derivative work of EULA-licensed code — incompatible with our OSS stack and redistribution model. Doing so contaminates the whole repo's license.
- ❌ **Cannot** redistribute UE source or tools outside Epic's channels.
- ✅ **Can** read UE source to *learn techniques* (the EULA does not patent the ideas; the algorithms are also published in SIGGRAPH/whitepapers). Re-implementing a *published technique* from scratch in clean-room Rust is legal and normal — that is what bevy_solari and bevy's virtual geometry already do.
- ✅ **Can** ship a separate UE-based showcase client (a normal UE licensee deliverable, royalty applies above $1M) that reads Civis world data. This is the sanctioned UE role.

**Verdict on wholesale reuse: not viable. Treat UE source as reference reading, never as a code donor.**

---

## 1b. Why reverse-engineering Lumen / Nanite / Chaos into Rust is impractical

Beyond license: these are **not portable algorithms, they are deeply C++-coupled subsystems**:

- **Nanite** — virtualized micro-polygon geometry: bespoke cluster DAG build (offline cook), a custom software rasterizer for sub-pixel triangles, visibility buffer, two-pass GPU occlusion culling, streaming, and material shading bound to UE's RHI/RDG render-graph and shader permutation system. Porting means reproducing the cook pipeline + RHI + RDG, not a function.
- **Lumen** — hybrid GI: screen-space + signed-distance-field tracing + (HW-RT) + surface cache + radiance caching + final gather, all wired to UE's mesh card system, GPU scene, and async compute scheduling. Tightly fused to UE's scene representation.
- **Chaos** — destruction/physics: geometry-collection fracture, field system, clustering, all integrated with UE's task graph, asset pipeline, and Niagara.

Each assumes UE's RHI, RDG render graph, GPU Scene, shader compiler, cook/streaming, and task graph. Re-hosting any one of them onto wgpu/Bevy is effectively re-implementing a third of UE. **Years of work, illegal to copy, and pointless when clean-room OSS equivalents already exist.**

---

## 1c. The realistic path — borrow the TECHNIQUES via OSS that already implement them

We don't need UE's code; we need the *capabilities*. Each UE marquee feature has an OSS/clean-room equivalent already in or near the Bevy 0.18 stack.

### Feature → OSS-equivalent → integration-effort table

| UE feature | Capability | OSS / clean-room equivalent (license) | Bevy 0.18 status | Integration effort |
|---|---|---|---|---|
| **Nanite** (virtualized geometry) | Sub-pixel LOD, meshlet DAG, GPU-driven cull, visibility buffer, impostors | **Bevy virtual geometry / `MeshletPlugin`** (MIT/Apache, in-tree, experimental); **`meshoptimizer`** via **`meshopt-rs`** (MIT) for meshlet build + `meshopt_SimplifySparse`; **`Scthe/nanite-webgpu`** (MIT) as a full reference impl incl. SW rasterizer + impostors; `METIS` for clustering | In-tree experimental, ~60–70% of Nanite's benefit; DAG quality is the active bottleneck; primary dev on intermittent break | **Med–High** — adopt in-tree meshlet renderer; borrow DAG/cluster ideas from nanite-webgpu; wrap meshopt for the cook |
| **Lumen** (realtime GI) | RT direct+indirect, ReSTIR DI/GI, world-space irradiance cache, denoise | **`bevy_solari`** (MIT/Apache, in-tree) — ReSTIR DI (1st bounce) + ReSTIR GI (2nd) + world cache (further bounces) + specular GGX, denoised; **`kajiya`** (MIT/Apache, Embark) as a reference RTGI design | In-tree in 0.18; needs DXR-class GPU (we target DX12U/DXR — fits); 0.18 added specular + balance-heuristic MIS + faster world cache | **Low–Med** — first-party; enable + tune. Strong fit for our DXR target |
| **DLSS / TSR** (upscaling) | Temporal super-resolution, ray-reconstruction | **`dlss_wgpu`** (Epic-of-NVIDIA SDK wrapped; standalone crate) wired into **`bevy_anti_alias`** (MIT/Apache, in-tree) — **DLSS + DLSS-RR ship since 0.17** | First-party in 0.18 | **Low** — already integrated; ship it. FSR/XeSS/MetalFX not yet integrated but infra exists |
| **Chaos** (destruction) | Fracture, rigid-body debris, fields | **Our voxel material-fluid CA** (the substrate) for emergent structural failure + **`Rapier`** (Apache, mature pure-Rust) or **`Avian`** (MIT/Apache) for rigid debris; **`Jolt`** via bindings if we need AAA-grade large-scene rigid bodies | Rapier/Avian mature for Bevy; Jolt is C++ (interop — see interop.md) | **Med** — destruction emerges from CA + physics; no monolithic "Chaos" port needed |
| **Niagara** (VFX) | GPU particles | `bevy_hanabi` (MIT/Apache, GPU particle system) | Ecosystem, mature | **Low–Med** |
| **MetaHuman / Nanite foliage** | Asset-class, not engine tech | Out of scope; asset/content problem | — | N/A |

**Bottom line for Unreal:** UE's three crown jewels each map to an in-tree or near-tree Bevy OSS equivalent that already delivers most of the value, legally, on our DX12U/DXR target. The work is *enable + tune + wrap meshopt*, not *port UE*.

---

## Sources
- [Realtime Raytracing in Bevy 0.18 (Solari)](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18/)
- [Bevy 0.18 release notes](https://bevy.org/news/bevy-0-18/)
- [Virtual Geometry in Bevy 0.16](https://jms55.github.io/posts/2025-03-27-virtual-geometry-bevy-0-16/)
- [Bevy virtual geometry discussion #10433](https://github.com/bevyengine/bevy/discussions/10433)
- [Scthe/nanite-webgpu](https://github.com/Scthe/nanite-webgpu)
- [zeux/meshoptimizer](https://github.com/zeux/meshoptimizer) · [meshopt-rs](https://github.com/gwihlidal/meshopt-rs) · [meshlet DAG simplify discussion #750](https://github.com/zeux/meshoptimizer/discussions/750)
- [bevy_anti_alias (DLSS)](https://crates.io/crates/bevy_anti_alias/0.18.0-rc.2) · [Bevy 0.17 notes (DLSS + dlss_wgpu)](https://bevy.org/news/bevy-0-17/)
- [Unreal Engine EULA](https://www.unrealengine.com/eula/unreal) · [UE licensing options](https://www.unrealengine.com/license)
