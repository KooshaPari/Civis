# C++ / C# Interop Strategy for Civis

**Principle:** stay pure-Rust by default (matches our MIT/Apache OSS stack, single toolchain, no marshalling). Reach for **C++ interop only to wrap a best-in-class NATIVE C++ library** that has no equal Rust crate. **C# interop is rare-to-never** — avoid.

---

## When C++ interop IS worth it

Wrap the C++ lib (don't reimplement) when it is (a) battle-tested AAA-grade, (b) no mature pure-Rust equivalent, (c) a stable, mostly-C-shaped API surface:

| Native C++ lib | Capability | Pure-Rust alternative? | Interop verdict |
|---|---|---|---|
| **meshoptimizer** (zeux) | Meshlet build, LOD simplify, `meshopt_SimplifySparse` for Nanite-style DAG | **`meshopt-rs`** already wraps it (FFI + idiomatic) | ✅ **Use `meshopt-rs`** — interop already done for us |
| **Recast / Detour** | Navmesh gen + pathfinding (UE/Unity/Godot standard) | **`rerecast`** = clean-room Rust port | ✅ **Prefer `rerecast`** (no FFI). Wrap C++ Recast only if rerecast lacks a needed feature |
| **JoltPhysics** (`JoltPhysics-C`) | AAA large-scene rigid-body / destruction debris | `Rapier`/`Avian` (pure Rust, mature). `jolt-rust` bindings exist but young | ⚠️ **Default Rapier/Avian.** Wrap Jolt-C only if profiling shows we need Jolt's large-scene perf |
| **PhysX** | AAA physics (GPU rigid bodies) | Rapier/Avian; `physx-rs` bindings exist | ⚠️ **Avoid unless required.** Heavy, NVIDIA-leaning; Rapier first |
| **NVIDIA DLSS SDK** | Upscaling | **`dlss_wgpu`** wraps the SDK; in `bevy_anti_alias` | ✅ **Already wrapped & in-tree** (0.17+) |
| **AMD FidelityFX (FSR), Intel XeSS** | Upscaling | Not yet in Bevy; SDKs are C++ | ⏳ **Wrap later** if we need FSR/XeSS on non-NVIDIA; Bevy temporal-upscale infra exists |
| **OpenVDB / NanoVDB** | Sparse-volume data structures (volumetric/SVO ideas) | Partial Rust SVO crates; no full VDB | ⚠️ **NanoVDB** (header-only, GPU-friendly) is the wrap candidate if we need true VDB; otherwise our SVO substrate suffices |

**Pattern: most of these are already wrapped by a Rust crate** (`meshopt-rs`, `dlss_wgpu`, `rerecast`, `physx-rs`, `jolt-rust`). Prefer the existing wrapper crate over hand-rolling FFI.

---

## Recommended interop toolchain (C++)

Ranked by preference:

1. **`cxx`** — *first choice for new bindings.* Safe, zero-overhead, you declare a shared bridge in Rust; ideal when the C++ API is small/stable or you control a thin C++ shim. Best safety/ergonomics.
2. **`autocxx`** — `cxx` + `bindgen` automation driven from existing C++ headers; auto-handles destructors, string/`UniquePtr` conversions. Use when the C++ surface is large and you want generated bindings with cxx-grade safety.
3. **`bindgen`** — lowest-level, oldest, most widely used (rust-lang org). Use for **C / C-shaped APIs** (e.g. `JoltPhysics-C`, NanoVDB C API, FidelityFX C headers). Pair with a hand-written safe Rust wrapper.
4. **Prefer a C wrapper layer:** when a lib offers a C API (`JoltPhysics-C`, Detour's C bindings), bind that with `bindgen` — far simpler than binding C++ ABI directly.

### Risk notes
- **Build complexity:** C++ deps drag in a C++ toolchain (MSVC on Windows / our DX12 target), `cc`/CMake build steps, longer CI, and platform-specific linking. Every wrapped lib raises build-fragility.
- **ABI / template pain:** `bindgen` can't handle C++ templates/overloads cleanly — that's why `autocxx`/`cxx` or a C shim exist. Heavy template libs (PhysX) are the worst.
- **Memory-safety boundary:** FFI is `unsafe`; ownership/lifetime bugs cross the boundary. Keep the wrapper thin and well-tested; expose only safe Rust.
- **License check each lib:** meshoptimizer (MIT), Recast/Detour (zlib), Jolt (MIT), PhysX (BSD-3), DLSS SDK (NVIDIA proprietary SDK, redistributable per its terms), OpenVDB (MPL-2.0/Apache). All compatible-to-manageable for an MIT/Apache app, but DLSS/NVIDIA SDK has its own redistribution terms.
- **Determinism: not a concern** for Civis ([emergence-charter]) — no need to reject a lib for nondeterminism.

---

## C# interop — avoid

- No native C# lib here is worth hosting the .NET CLR (hostfxr/`netcorehost`) inside a Bevy process: adds a GC runtime, marshalling, and a deploy dependency.
- Unity/UE C# assets are license-locked to their engines anyway (see unity.md, unreal.md).
- **Recommendation: do not do C# interop.** If a capability seems C#-only, the underlying value is almost always a native C++ lib you can wrap directly instead.

---

## Sources
- [autocxx](https://google.github.io/autocxx/) · [cxx.rs](https://cxx.rs/context.html) · [bindgen + cxx ecosystem tour (eShard)](https://www.eshard.com/blog/rust-cxx-interop) · [KDAB: mixing Rust & C++](https://www.kdab.com/mixing-c-and-rust-for-fun-and-profit-part-3/)
- [meshopt-rs](https://github.com/gwihlidal/meshopt-rs) · [meshoptimizer](https://github.com/zeux/meshoptimizer)
- [rerecast](https://github.com/janhohenheim/rerecast)
- [dlss_wgpu / bevy_anti_alias (Bevy 0.17 notes)](https://bevy.org/news/bevy-0-17/)
- [Avian physics](https://joonaa.dev/blog/07/avian-0-2) · [Rust game physics engines overview](https://rodneylab.com/rust-game-physics-engines/)
