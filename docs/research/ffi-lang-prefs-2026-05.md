# FFI / Native Acceleration Language Preference

Snapshot: 2026-05-21. Sources below use current repo/package metadata and official docs.

## Score Matrix

Scores are 1-10, higher is better for DINOForge.

| Use case | Rust | Zig | Mojo | Recommendation |
|---|---:|---:|---:|---:|
| Rust lib consumed by `.NET 8.0` + `netstandard2.0` / Unity Mono | 10 | 7 | 2 | Rust |
| Asset-pipeline kernels (mesh decimation, BVH, transcode, import helpers) | 9 | 7 | 3 | Rust |
| Small, self-contained native utility with a strict C ABI | 8 | 9 | 3 | Zig |
| Experimental/prototyping only | 9 | 7 | 5 | Rust or Zig |

## Production Call

For DINOForge production, the SOTA path is still:

1. Rust `cdylib` with explicit `extern "C"` exports.
2. Consume it from C# with `DllImport` on `netstandard2.0` / Unity Mono, and `LibraryImport` where the host is .NET 8+.
3. Use `csbindgen` when you want generated C# declarations and Unity-friendly glue.

Why this wins:

- It keeps the ABI simple and stable across Windows/Linux/macOS.
- It is the least risky path for Unity Mono, which wants a plain native DLL/SO/dylib boundary.
- It works cleanly for both long-lived runtime DLLs and short-lived tool subprocesses.

## Rust FFI Options

- `csbindgen` is the strongest pragmatic choice for .NET interop ergonomics right now: v1.9.8, 915 stars, latest commit 2026-05-21.
- Direct C ABI is still the most reliable shipping contract when you care about Unity Mono and long-term ABI stability.
- `uniffi-bindgen-cs` is active but younger: 172 stars, latest release `v0.10.0+v0.29.4` (2025-08-22). Good for experimentation, not my first choice for a production Unity bridge.
- Native AOT exports are useful when .NET is the native producer. For DINOForge’s direction, Rust is the producer, so they are not the primary fit.

## Zig FFI

- Zig is viable for tiny native kernels because `export fn` / C ABI export is straightforward.
- It is a reasonable choice for isolated mesh/BVH helpers, but the ecosystem for asset-pipeline plumbing is thinner than Rust’s.
- For Unity Mono and cross-platform packaging, Zig is “works if kept small,” not the best default.

## Mojo FFI

- Mojo is still too early for DINOForge production ABI work.
- Latest public release is `MAX 26.3 / Mojo 1.0.0b1` (2026-05-07), which is not a “boring stable ABI” signal.
- Modular’s docs show C-ABI export/FFI capability, but Windows support is still documented through WSL in current public docs.
- Recommendation: defer unless you want a very small experimental spike.

## Asset Pipeline Libraries

- `meshoptimizer` is the safest decimation/BVH-adjacent baseline: v1.1, 7.7k stars, very active upstream.
- `meshopt` (Rust crate) is the best Rust-native option if you want to stay in one language: v0.6.2, active, but smaller ecosystem than upstream C++.
- `image` crate is current and healthy for PNG/DDS-class work: v0.25.10, 127M downloads. It is fine for PNG/DDS, but KTX usually needs an additional crate or a different pipeline.
- `AssimpNet` is still more credible than `assimp-rs` for broad-format import. `assimp-rs` is effectively dead (`0.0.1`, last active 2015), so do not build new production work on it.

## Top 3 Wins

1. Rust `cdylib` + C ABI + `csbindgen` for the asset-pipeline/bridge acceleration layer.
2. `meshoptimizer` for decimation-heavy native work, with Rust `meshopt` only if you want a pure-Rust implementation.
3. `image` for PNG/DDS decode/encode, with a separate KTX path only where needed.

## Top 3 To Avoid

1. Mojo as the primary production ABI layer.
2. `assimp-rs` for new production import work.
3. UniFFI C# as the main Unity/Mono bridge when a plain C ABI is sufficient.

## Risk Summary

- Rust: lowest ABI risk, medium build complexity (`cargo` + `dotnet` hybrid), best production fit.
- Zig: low runtime overhead, but higher ecosystem risk for asset-pipeline libraries.
- Mojo: highest product risk and highest ABI/tooling uncertainty; defer.

## Sources

- .NET native interop: https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/interop
- .NET P/Invoke interop: https://learn.microsoft.com/en-us/dotnet/standard/native-interop/pinvoke
- Unity native plugin / DllImport docs: https://docs.unity3d.com/Manual/PluginsForDesktop.html
- Zig release + docs: https://github.com/ziglang/zig and https://ziglang.org/documentation/master/
- Mojo release/docs: https://github.com/modular/mojo and https://docs.modular.com/mojo/
- UniFFI / C# bindings: https://github.com/mozilla/uniffi-rs and https://github.com/NordSecurity/uniffi-bindgen-cs
- `csbindgen`: https://github.com/Cysharp/csbindgen
- `meshoptimizer`: https://github.com/zeux/meshoptimizer
- Rust `meshopt`: https://crates.io/crates/meshopt
- Rust `image`: https://crates.io/crates/image
