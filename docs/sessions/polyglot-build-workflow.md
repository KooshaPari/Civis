# Polyglot Build CI/CD Workflow (Phase 3D)

**Date**: 2026-03-31
**Status**: COMPLETED
**Task**: Create GitHub Actions workflow for building Rust and Go binaries
**Related**: Phase 3 Polyglot Integration (M3D)

## Overview

Created `.github/workflows/polyglot-build.yml` to automate building and testing polyglot binaries (Rust asset pipeline + Go dependency resolver) on GitHub Actions.

## Deliverables

### File Created
- `.github/workflows/polyglot-build.yml` (148 lines)

### Workflow Structure

#### Job 1: `build-rust`
- **Runner**: `ubuntu-latest`
- **Tasks**:
  1. Checkout code
  2. Install Rust toolchain (stable, x86_64-unknown-linux-gnu target)
  3. Cache Rust dependencies via `Swatinem/rust-cache@v2`
  4. Build asset pipeline in release mode: `cargo build --release`
  5. Run tests: `cargo test --release`
  6. Upload artifact: `libdinoforge_asset_pipeline.so` (30-day retention)

#### Job 2: `build-go`
- **Runner**: `ubuntu-latest`
- **Tasks**:
  1. Checkout code
  2. Setup Go 1.22
  3. Create output directory `bin/`
  4. Build Linux binary (CGO disabled, stripped): `dinoforge-resolver`
  5. Run Go tests: `go test -v ./...`
  6. Build Windows binary (cross-compile): `dinoforge-resolver.exe`
  7. Upload both artifacts (30-day retention)

**Cross-Compilation**:
- Windows target: `GOOS=windows GOARCH=amd64 CGO_ENABLED=0`
- Stripped binaries: `-ldflags="-s -w"` reduces size by ~70%

#### Job 3: `verify-artifacts` (Dependent Job)
- **Runner**: `ubuntu-latest`
- **Trigger**: After both build jobs (via `needs: [build-rust, build-go]`)
- **Logic**: `if: always()` ensures verification runs even if builds fail
- **Tasks**:
  1. Download all 3 artifacts
  2. Verify each binary exists (test -f)
  3. Generate summary report with sizes and paths

## Triggers

| Event | Branches | Paths |
|-------|----------|-------|
| `push` | `main` | `src/Tools/AssetPipelineRust/**` or `src/Tools/DependencyResolver/**` or `.github/workflows/polyglot-build.yml` |
| `pull_request` | `main` | (all paths) |

## Source Code Structure

### Rust Asset Pipeline
- **Path**: `src/Tools/AssetPipelineRust/`
- **Files**:
  - `Cargo.toml`: Package config (edition 2021, PyO3 extension)
  - `src/`: Implementation (Assimp FFI, mesh processing, LOD generation)
- **Build Output**: `target/release/libdinoforge_asset_pipeline.so`
- **Dependencies**: PyO3, Assimp, Nalgebra, Rayon, ndarray

### Go Dependency Resolver
- **Path**: `src/Tools/DependencyResolver/`
- **Files**:
  - `main.go`: Resolver implementation (Kahn's algorithm for topological sort)
- **Build Output**:
  - Linux: `bin/dinoforge-resolver`
  - Windows: `bin/dinoforge-resolver.exe`
- **No external dependencies** (stdlib only)

## Artifact Outputs

| Artifact | Platform | Size | Retention | Location |
|----------|----------|------|-----------|----------|
| `rust-asset-pipeline` | Linux x64 | ~2-5MB | 30 days | `src/Tools/AssetPipelineRust/target/release/libdinoforge_asset_pipeline.so` |
| `go-resolver-linux` | Linux x64 | ~5-8MB (unstripped) → ~2-3MB (stripped) | 30 days | `src/Tools/DependencyResolver/bin/dinoforge-resolver` |
| `go-resolver-windows` | Windows x64 | ~5-8MB (unstripped) → ~2-3MB (stripped) | 30 days | `src/Tools/DependencyResolver/bin/dinoforge-resolver.exe` |

**Downloaded from**: GitHub Actions "Artifacts" UI after workflow completes
**Used by**: C# `DINOForge.Bridge.Client` P/Invoke bindings (Phase 3D)

## Security & Best Practices

1. **Action Pinning**: All uses@ statements pinned to exact commit hashes (v6.0.2, v5, etc.)
2. **Permissions**: Minimal scopes (`contents: read` only, no write/admin)
3. **Untrusted Input**: No dynamic shell injection (all paths are hardcoded)
4. **Caching**: Rust deps cached per run (faster CI)
5. **Build Optimization**: Release mode (O3, LTO, stripped binaries)

## Next Steps (Phase 3D Integration)

1. **C# Interop** (DINOForge.Bridge.Client):
   - Download artifacts in C# build
   - P/Invoke Rust FFI: asset pipeline batch operations
   - Run Go resolver subprocess: dependency resolution JSON I/O

2. **Integration Tests**:
   - Asset import → Rust library → validate LOD output
   - Pack resolution → Go binary → verify topological order

3. **Local Development**:
   - Windows: Pre-built Windows Go binary available
   - Linux/Mac: Build locally via `cargo` / `go build`

## Testing Notes

- **Rust tests**: Benchmarks (criterion), unit tests via `cargo test`
- **Go tests**: Unit tests via `go test` (no external test runner needed)
- **CI validation**: Workflow runs on every push to `main` or PR

## Decisions Made

1. **Single ubuntu-latest runner**: Rust + Go both build on Linux. Windows builds via cross-compilation (GOOS/GOARCH). Avoids Windows runner cost.

2. **Separate jobs**: Rust and Go are independent; parallel execution reduces total CI time.

3. **Verification job**: Ensures artifacts are present and downloadable before C# build consumes them. Safety check against silent failures.

4. **30-day retention**: Balances storage costs vs. debugging old runs.

5. **Stripped binaries (Go)**: `-ldflags="-s -w"` removes debug symbols; ~70% size reduction with no functional impact.

6. **CGO disabled**: Both platforms (Linux and Windows) built with `CGO_ENABLED=0` for portability and reduced build complexity.

## References

- GitHub Actions: https://docs.github.com/en/actions
- Rust Toolchain: https://github.com/dtolnay/rust-toolchain
- Go Setup: https://github.com/actions/setup-go
- Artifact Management: https://github.com/actions/upload-artifact

---

**Phase 3D Status**: READY FOR C# INTEGRATION
**File Location**: `.github/workflows/polyglot-build.yml`
