# Language Optimality Audit

**Scope:** Civis sim core is Rust. This memo evaluates where other languages are a better fit than Rust for specific surfaces, and where they are not.

**Bottom line:** keep Rust as the default implementation language for core simulation and most backend logic. Use other languages only where they unlock a platform boundary, a toolchain requirement, or a materially better developer/runtime model for a narrow subsystem.

## Summary Table

| Language | Best fit in Civis | Recommendation |
|---|---|---|
| CUDA/PTX | GPU hot loops, not general game logic | Use selectively for proven hotspots; prefer Rust host code plus CUDA kernels, not raw PTX |
| Zig | Thin C ABI glue, build tooling | Not worth replacing Rust FFI shims broadly |
| Go | Concurrent ops tooling, small daemons | Use for standalone infra tools if it reduces complexity; not for core services |
| C++ | Unreal engine modules and unavoidable SDKs | Use only where the engine or vendor API requires it |
| Mojo | Experimental AI/ML kernels and Python-adjacent prototypes | Evaluate only in a sandbox; too immature for core dependencies |
| C# | Unity client scripting | Use as planned |
| TypeScript | Web dashboard and UI orchestration | Keep TS; move only hot compute to Rust/WebAssembly |
| Python 3.14 | MCP servers, automation, LLM pipelines | Worth upgrading for tooling; not a reason to rewrite systems |
| Python 7 preview | N/A | Ignore; there is no meaningful public Python 7 target to plan against |
| WGSL / HLSL / GLSL / MSL | Shader source and backend targets | Standardize on WGSL as canonical for shared shader logic, with backend-specific emission where needed |

## 1. CUDA/PTX

### Use case

GPU compute kernels for:

- voxel operations
- agent pathfinding
- terrain generation
- particle simulation

### Performance benefit over Rust

Rust is the host language; it is not the compute backend. For GPU work, the real choice is between:

- Rust host + CUDA kernels
- Rust host + Rust GPU kernels via `cuda_std` / related NVPTX tooling
- Rust host + external CUDA C++ kernels
- raw PTX for hand-tuned assembly-like kernels

CUDA is the mature NVIDIA-native route for throughput-heavy parallel kernels. Raw PTX gives maximum control, but only pays off when you have already hit a kernel-level bottleneck that higher-level CUDA cannot fix.

### Integration path

- Keep Rust as the host/orchestration layer.
- Start with Rust-side abstractions where possible, then offload hot kernels to CUDA only when profiling proves it.
- Use Rust CUDA tooling such as `cuda_std` for kernel code when the kernel surface can stay in Rust and NVPTX support is acceptable.
- Use CUDA C++ for kernels that need better library compatibility, debugging support, or existing CUDA ecosystem reuse.
- Reserve raw PTX for last-mile tuning of a small number of kernels.

### Recommendation

**Use CUDA selectively; do not standardize on raw PTX.**  
For Civis, the default should be Rust host code plus CUDA kernels only for proven hotspots. If a kernel can stay in Rust via `cuda_std`, that is usually better than switching to PTX. If the kernel needs the full NVIDIA ecosystem, use CUDA C++ rather than hand-writing PTX.

### Notes

- NVIDIA’s official CUDA guide positions CUDA as the standard GPU programming model and emphasizes CUDA C++ as the primary authoring surface.
- Rust’s `cuda_std` crate is a curated Rust GPU-kernel layer, but it is NVPTX-specific and not a general replacement for CUDA’s native ecosystem.

Sources: [CUDA Programming Guide](https://docs.nvidia.com/cuda/cuda-programming-guide/index.html), [cuda_std crate docs](https://docs.rs/cuda_std/latest/cuda_std/).

## 2. Zig

### Use case

Potential replacement for C FFI glue, including:

- cbindgen-generated shims
- tiny ABI adapters for Unreal/Unity
- small build helpers and cross-platform utilities

### Performance benefit over Rust

Very little. Zig can reduce glue-layer friction because it can emit C ABI-compatible code naturally and has a simpler low-level story than C. But Rust already does C ABI interop well, and the glue layer is not where Civis should be spending language complexity budget.

### Integration path

- Keep Rust for the implementation.
- Use `extern "C"` interfaces, `bindgen`, `cbindgen`, or handwritten ABI shims where needed.
- If a shim becomes a standalone utility with lots of low-level platform code, Zig can be considered as an implementation language for that tool only.

### Recommendation

**Do not replace Rust FFI glue with Zig by default.**  
Zig is reasonable for isolated tooling or edge-case native wrappers, but it is not worth introducing as the standard shim language for Civis. The maintenance cost of a third systems language outweighs the marginal gain.

### Notes

- Zig’s documentation explicitly calls out C-ABI compatibility and careful target/ABI handling as first-class concerns.

Source: [Zig documentation](https://ziglang.org/documentation/master/).

## 3. Go

### Use case

Backend services or tooling where the primary need is:

- simple concurrent fan-in/fan-out
- small static binaries
- easy ops scripting for services and monitors
- CI helpers or release automation

### Performance benefit over Rust

Go’s goroutines are lightweight and make concurrent service code easy to express. That is a developer-experience win more than a raw performance win. Rust async can match or beat Go on throughput and memory efficiency when implemented well, but Go is often simpler for short-lived ops tools or multiplexed service logic.

### Integration path

- Keep Rust for the main backend and data-plane logic.
- Use Go only for standalone operational tools where goroutines and a simple deployment story are more valuable than Rust’s control.
- Keep service boundaries coarse and over RPC if Go is introduced, so the language boundary stays obvious.

### Recommendation

**Go is optional, not strategic.**  
Use it only for infra-adjacent tools if it lowers implementation time. Do not migrate backend services away from Rust just to get goroutines. Civis does not need a second service-language unless a specific tool is markedly easier in Go.

### Notes

- Go’s official docs describe goroutines as lightweight threads and emphasize concurrency as a core language feature.
- Go itself cautions that concurrency is not the same as parallelism, so goroutines are not a universal performance trump card.

Sources: [Go tour: concurrency](https://go.dev/tour/concurrency), [Effective Go](https://go.dev/doc/effective_go?lang=en), [Go FAQ](https://go.dev/doc/faq?source=post_page-----a4e575dff860--------------------).

## 4. C++

### Use case

Engine interop, especially:

- Unreal Engine modules
- third-party SDKs that only expose C++
- places where the engine’s native extension model expects C++

### Performance benefit over Rust

Usually none at the algorithmic level. C++ is useful because it is the native language of the engine or SDK, not because it is inherently better than Rust for Civis logic.

### Integration path

- Keep C++ at the narrow boundary where Unreal requires it.
- Move actual simulation and data logic behind a Rust-facing API whenever possible.
- Avoid expanding C++ beyond the engine integration layer.

### Recommendation

**Use C++ only where the engine or vendor forces it.**  
This is the correct place for C++ in Civis: glue, module entrypoints, and APIs that cannot realistically be expressed otherwise. It should not become a second implementation language for the sim.

## 5. Mojo

### Use case

Potential AI/ML workloads:

- LLM inference
- embedding generation
- GPU-heavy numerical kernels
- Python-adjacent ML prototyping

### Performance benefit over Rust

Mojo’s value proposition is a Python-friendly syntax with compiled performance and GPU support. That makes it attractive for ML workloads that want tighter Python interop than Rust usually provides. But it is still an emerging platform, and “promising” is not the same as “operationally safer than Rust.”

### Integration path

- Treat Mojo as a research lane, not a production default.
- If used, keep it behind a very small service or module boundary.
- Prefer Python interop for experimentation, but do not make Mojo the source of truth for critical game or infrastructure logic.

### Recommendation

**Worth a sandbox evaluation only.**  
For an on-device Firepass alternative, Mojo is interesting if the main goal is fast iteration on GPU-heavy ML code with Python interoperability. It is not yet a justified replacement for Rust or a mature inference stack in Civis.

### Notes

- Modular’s docs describe Mojo as compiled, Python-interoperable, and GPU-capable across vendors, but the ecosystem is still actively evolving.

Sources: [Mojo Python interoperability](https://docs.modular.com/mojo/manual/python/), [Mojo GPU programming fundamentals](https://docs.modular.com/mojo/manual/gpu/fundamentals), [Mojo system requirements](https://docs.modular.com/mojo/requirements/).

## 6. C#

### Use case

- Unity client scripting

### Performance benefit over Rust

None in the abstract. The benefit is platform fit: Unity expects C#.

### Integration path

- Use C# as the Unity-facing scripting and gameplay layer.
- Keep shared logic in Rust only where the Unity integration model supports a clean boundary.

### Recommendation

**Use C# as planned.**  
This is a platform requirement, not an optimization gamble.

## 7. TypeScript

### Use case

- web dashboard
- client-side UI orchestration
- product surface logic

### Performance benefit over Rust

TypeScript is not a performance language; it is a productivity and ecosystem choice for the web. Rust only wins if you have CPU-heavy logic that genuinely belongs in a compiled module.

### Integration path

- Keep the dashboard and UI in TypeScript.
- Move only hot compute, strict data transforms, or serialization-heavy logic to Rust via WebAssembly or a backend API.
- Avoid rewriting normal UI code in Rust unless it is clearly a hotspot.

### Recommendation

**Do not plan a broad TS-to-Rust migration.**  
The correct split is TypeScript for UI/product logic and Rust for heavy compute or shared core logic when the boundary is justified.

## 8. Python 3.14

### Use case

- MCP servers
- tooling scripts
- LLM pipelines
- local automation

### Performance benefit over Rust

Python 3.14 improves runtime characteristics, but it does not change Python’s role as a high-productivity orchestration language. The real gains are:

- free-threaded mode is officially supported and improved
- official macOS and Windows binaries include an experimental JIT
- some standard-library and runtime improvements reduce overhead

The free-threaded build still has compatibility caveats, and the JIT is explicitly experimental.

### Integration path

- Upgrade tooling and automation first.
- Use the default GIL build unless a dependency stack is confirmed compatible with free-threaded Python.
- Treat the JIT as an experiment, not a production assumption.

### Recommendation

**Yes, upgrade tooling to Python 3.14 where dependencies permit.**  
This is worth doing for MCP servers and scripts, but not because Python becomes a replacement for Rust. The upside is mostly concurrency and runtime hygiene, not a dramatic performance leap.

### Notes

- Python 3.14 official docs describe free-threaded mode as supported and improved, with an estimated 5-10% single-thread penalty in free-threaded builds.
- The experimental JIT is available in official macOS and Windows binaries, but the docs say it is not recommended for production use.

Sources: [What’s new in Python 3.14](https://docs.python.org/3/whatsnew/3.14.html), [Python support for free threading](https://docs.python.org/3/howto/free-threading-python.html), [Python 3.14 docs](https://docs.python.org/3.14/).

## 9. Python 7 preview

### Use case

None.

### Performance benefit over Rust

None, because there is no meaningful public Python 7 preview to evaluate for Civis planning.

### Integration path

None.

### Recommendation

**Ignore.**  
Do not plan around a Python 7 preview. The practical upgrade axis today is Python 3.14 and its compatibility story.

## 10. WGSL / HLSL / GLSL / MSL

### Use case

- WGSL: WebGPU/web-facing render and compute shaders
- HLSL: Unreal / DirectX-centric native passes
- GLSL: legacy OpenGL or import/export compatibility
- MSL: Apple/Metal native passes

### Performance benefit over Rust

None directly. These are shader languages, not host languages. The question is not “faster than Rust” but “best match for the target graphics backend.”

### Integration path

- Standardize shader authoring around WGSL where the goal is shared logic across platforms and web delivery.
- Emit or translate to backend-specific formats where required:
  - HLSL for Unreal / DirectX targets
  - MSL for Metal targets
  - GLSL only when OpenGL compatibility is unavoidable
- Keep platform-specific differences at the edges of the shader pipeline, not in gameplay code.

### Recommendation

**Use WGSL as the canonical source language for shared shader logic, with backend-specific output as needed.**  
This is the cleanest fit for Civis because the web stack already benefits from WebGPU/WGSL, and it keeps the source of truth in one place. Use HLSL or MSL only when a platform-native shader path is unavoidable. Avoid GLSL as a first choice unless you are supporting an old OpenGL path.

### Notes

- WGSL is the WebGPU shading language standard.
- Apple’s Metal docs treat MSL as the definitive shader language for Metal.
- Microsoft’s HLSL docs describe HLSL as the shader language for Direct3D.

Sources: [WGSL spec](https://www.w3.org/TR/WGSL/), [Apple Metal resources](https://developer.apple.com/metal/resources/), [HLSL compilation docs](https://learn.microsoft.com/en-us/windows/win32/direct3dhlsl/dx-graphics-hlsl-part1), [OpenGL Shading Language spec](https://registry.khronos.org/OpenGL/specs/gl/GLSLangSpec.4.60.html).

## Final Recommendation

If Civis wants an optimal multi-language strategy, the stack should look like this:

1. **Rust** remains the default for simulation, backend logic, and shared core code.
2. **CUDA** is the only serious candidate for high-value GPU compute, but only for measured hotspots.
3. **C++** stays at Unreal and vendor-API boundaries.
4. **TypeScript, C#, and Python 3.14** remain the productivity languages for web, Unity, and tooling respectively.
5. **WGSL** should be the canonical shader source for shared GPU rendering logic.
6. **Zig, Go, and Mojo** are niche options, not strategic defaults.

The practical optimization rule for Civis is simple: introduce a second language only when it clearly reduces boundary friction, platform friction, or kernel-level runtime cost. Otherwise, Rust is the better default because it keeps ownership, safety, and maintenance centralized.
