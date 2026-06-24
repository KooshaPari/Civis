# Unity — Parity Path for Civis

**Verdict up front: nothing to reverse-engineer.** Unity is fully proprietary, closed-source (no source-available tier comparable to UE), and C#/managed. There is no legal or practical donor code. The value is **conceptual** — Unity validated patterns we already get natively in Bevy/Rust.

---

## Concepts to borrow (already covered by our stack)

| Unity concept | What it gives | Civis (Bevy 0.18 / Rust) equivalent | Action |
|---|---|---|---|
| **DOTS / ECS** (Entities) | Data-oriented entity sim at scale | **Bevy is ECS-native** — archetypal storage, system scheduling, change detection | None — Bevy already exceeds the ergonomics; just adopt DOTS-style data layout discipline |
| **Burst compiler** (SIMD-JIT of C# subset) | Near-native hot loops | **Rust compiles native** with LLVM autovectorization; `std::simd` / `wide` / `glam` for explicit SIMD | None — Rust is the native baseline Burst is trying to reach |
| **Job System** | Safe data-parallel jobs | `bevy_tasks` + parallel ECS system execution; `rayon` for data-parallel loops | None |
| **Shader Graph** | Node-based material authoring | Bevy material system + `bevy`'s shader/material APIs; node-graph tooling is an ecosystem editor concern, not a runtime gap | Defer (editor tooling, low priority) |
| **Addressables / streaming** | Asset streaming | Our SVO + dense-leaf-chunk streaming + `bevy_asset` | None — already built ([sio2-and-voxel-baselines]) |
| **NavMesh (Unity uses Recast under the hood)** | Pathfinding | **`bevy_rerecast` / `rerecast`** (clean-room Rust Recast port) — see godot.md | Adopt rerecast (shared with Godot finding) |

---

## C# interop — rare, avoid

Unity's ecosystem is **C#/managed**. C# interop into a Rust process is only worth it for a *specific, irreplaceable C#-only asset/library*, and that case essentially does not arise here:

- Unity Asset Store content is licensed for use *inside Unity*, not for extraction into a Rust engine — license-incompatible, same trap as UE source.
- Unity's engine runtime (the thing of value) is closed C++ behind a C# API; you cannot get at it.
- Hosting the .NET CLR (via `netcorehost` / hostfxr) inside a Bevy process to call one C# lib adds a GC runtime, marshalling cost, and a deployment dependency — almost never justified.

**Recommendation:** treat Unity as **idea validation only**. No code, no interop. If a *specific* best-in-class capability exists only as a native C++ lib (PhysX, Recast/Detour, meshoptimizer), wrap the **C++** lib directly (see interop.md) — that path is unrelated to Unity.

---

## Sources
- [Bevy 0.18 release notes](https://bevy.org/news/bevy-0-18/)
- [rerecast — Rust port of Recast](https://github.com/janhohenheim/rerecast)
- (Unity DOTS/Burst/Shader Graph are vendor docs; the parity claims above rely on Bevy being ECS-native and Rust being natively compiled — established facts.)
