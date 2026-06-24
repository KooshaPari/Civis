# Other OSS Engines to Mine for Parity

Engines/renderers whose **OSS code or designs** we can legally study, fork, or depend on for AAA-parity pieces. Ranked roughly by usefulness to a Bevy/Rust/wgpu stack.

| Project | License | Language / stack | What it gives Civis | Reuse mode |
|---|---|---|---|---|
| **Veloren** | GPL-3.0 | Rust, wgpu, custom | Production **voxel terrain + chunk streaming + LOD** at planet scale, plus weather/world-sim. Closest peer to our SVO+chunk substrate | **Study/design reference.** ⚠️ **GPL-3.0** — do NOT copy code into our MIT/Apache tree (license-incompatible). Learn algorithms, reimplement clean. Already flagged in [sio2-and-voxel-baselines] |
| **rend3** | MIT/Apache | Rust, **wgpu** | Customizable wgpu renderer (render-graph, GPU-driven, PBR). Same GPU backend as Bevy | **Borrow patterns / crates.** License-compatible. Reference for render-graph + GPU-driven design; Bevy's own renderer is primary |
| **O3DE** (Atom) | **Apache-2.0** | C++ | **Atom**: modular, data-driven, multithreaded PBR renderer; Forward+ and Deferred; fully data-driven pipeline. AAA-grade, fee-free, Apache | **Design reference + selective port.** Apache is compatible. C++ → would need interop or clean reimpl; use as a *design* source for Forward+/pipeline data model, not a runtime dep |
| **Flax** | Source-available (custom, mostly free) | C++/C# | Full engine, modern renderer; readable C++ | **Reference only.** License is not OSS — read for ideas, do not vendor |
| **Stride** (formerly Xenko) | MIT | C# / .NET | MIT engine with a clean renderer + ECS-ish design | **Design reference.** C# stack; algorithms portable, code not (different language/runtime) |
| **Ambient** | MIT/Apache | Rust, wgpu, WASM | Multiplayer-first Rust/wgpu runtime with ECS + networking; project now largely dormant | **Borrow networking/ECS patterns.** Compatible license; mine for multiplayer/data-model ideas |
| **Bevy ecosystem** | MIT/Apache | Rust | `bevy_solari` (RTGI), in-tree meshlet/virtual-geometry, `bevy_anti_alias` (DLSS), `bevy_hanabi` (VFX), `bevy_rerecast` (navmesh), `avian`/`bevy_rapier` (physics), voxel crates (`block-mesh-rs`, `bevy_voxel_world`, `building-blocks`) | **Primary dependency surface.** Our default — wrap > handroll |

## Takeaways
- **License gate matters:** GPL (Veloren) = study-only; Apache/MIT (rend3, O3DE, Ambient, Stride, Bevy) = study + reuse subject to language.
- **Best direct-dependency matches** are all in the **Bevy/wgpu/Rust** circle (rend3, Bevy ecosystem) — no interop, no relicense friction.
- **Best AAA-design references** are **O3DE Atom** (Apache, Forward+ data-driven pipeline) and **Veloren** (planet-scale voxel streaming, design only).

## Sources
- [O3DE GitHub (Apache-2.0)](https://github.com/o3de/o3de/) · [Atom renderer guide](https://docs.o3de.org/docs/atom-guide/)
- [rend3 (MIT/Apache)](https://github.com/BVE-Reborn/rend3/blob/trunk/LICENSE.APACHE)
- [bevy_solari 0.18](https://jms55.github.io/posts/2025-12-27-solari-bevy-0-18/) · [Bevy 0.18](https://bevy.org/news/bevy-0-18/)
