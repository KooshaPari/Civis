# Polyglot Build System

DINOForge uses **4 languages + Python** for different layers:

## Language Overview

| Language | Purpose | Location | Build | Test |
|----------|---------|----------|-------|------|
| **C#** | Core SDK, runtime, interop | `src/SDK`, `src/Runtime`, `src/Domains`, `src/Bridge` | `dotnet build` | `dotnet test` |
| **Rust** | Asset pipeline, PyO3 FFI | `src/Tools/AssetPipelineRust` | `cargo build --release` | `cargo test --release` |
| **Go** | Pack dependency resolver | `src/Tools/DependencyResolver` | `go build` | `go test -v ./...` |
| **Zig** | Mesh decimation, spatial BVH | `src/Tools/AssetPipelineZig` | `zig build -O ReleaseFast` | `zig test src/root.zig` |
| **Python** | MCP server, automation | `src/Tools/DinoforgeMcp` | `pip install -e .` | `pytest tests/` |

## Quick Start

**Using Task runner** (recommended):
```bash
# Install task CLI (https://taskfile.dev/)
task build:all     # Build everything
task test:all      # Test everything
task precommit     # Full validation
```

**Manual per-language**:
```bash
# C# (.NET)
dotnet build src/DINOForge.sln -c Release
dotnet test src/DINOForge.sln

# Rust
cd src/Tools/AssetPipelineRust && cargo build --release && cargo test --release

# Go
cd src/Tools/DependencyResolver && go build . && go test -v ./...

# Zig
cd src/Tools/AssetPipelineZig && zig build -O ReleaseFast && zig test src/root.zig

# Python
cd src/Tools/DinoforgeMcp && pip install -e ".[dev]" && pytest tests/
```

## Build Order

Build in this order to respect dependencies:

1. **Rust** → `libdinoforge_asset_pipeline.so` (asset pipeline)
2. **Go** → `dinoforge-resolver` (pack dependency resolver)
3. **Zig** → `dinoforge_asset_pipeline_zig.dll` (mesh decimation)
4. **Python** → `dinoforge_mcp` (MCP server, depends on above binaries)
5. **C#** → `DINOForge.sln` (links polyglot binaries via P/Invoke)

## CI/CD Pipeline

GitHub Actions workflows:

| Workflow | Purpose | Trigger | Languages |
|----------|---------|---------|-----------|
| `.github/workflows/ci.yml` | Main build + test | Push to main, PRs | C# (.NET) |
| `.github/workflows/polyglot-build.yml` | Parallel polyglot builds | Path filters (Rust, Go, Zig) | Rust, Go, Zig, PlayCUA |
| `.github/workflows/release.yml` | NuGet package publishing | Git tags | C# |

### polyglot-build.yml Details

**Jobs** (run in parallel):
1. **build-rust**: Builds `libdinoforge_asset_pipeline.so` (Linux)
2. **build-go**: Builds resolver for Linux + Windows (cross-compile)
3. **build-zig**: Builds spatial indexing library (Windows + Linux)
4. **verify-artifacts**: Downloads all, validates, reports sizes

**Artifact Retention**: 30 days

**Triggers**:
- Filters on `src/Tools/AssetPipelineRust/**`, `src/Tools/DependencyResolver/**`, `src/Tools/AssetPipelineZig/**`
- Runs on push to main, all PRs

## Artifact Management

| Component | Artifact | Registry | Consumed By |
|-----------|----------|----------|-------------|
| C# SDK | `DINOForge.SDK.*.nupkg` | NuGet.org | Downstream mods, users |
| Bridge Protocol | `DINOForge.Bridge.Protocol.*.nupkg` | NuGet.org | Bridge.Client |
| Bridge Client | `DINOForge.Bridge.Client.*.nupkg` | NuGet.org | Game bridge integration |
| Rust lib | `libdinoforge_asset_pipeline.so` | GitHub Releases | C# P/Invoke (PackCompiler) |
| Go resolver | `dinoforge-resolver` (binary) | GitHub Releases | C# subprocess spawn |
| Zig lib | `dinoforge_asset_pipeline_zig.dll` | GitHub Releases | C# P/Invoke (mesh decimation) |
| Python MCP | `dinoforge-mcp` (package) | PyPI (future) | Claude Code, game automation |

## Development Workflows

### Adding a C# Feature

1. Add code to `src/SDK`, domain project, or bridge layer
2. Add unit tests to `src/Tests`
3. Run `dotnet build && dotnet test`
4. Create PR (CI runs automatically)

### Adding a Rust Optimization

1. Modify `src/Tools/AssetPipelineRust/src/`
2. Run `cargo test --release` and `cargo bench`
3. Verify C# P/Invoke bindings still work
4. Create PR (polyglot-build.yml triggers)

### Adding a Go Feature

1. Add code to `src/Tools/DependencyResolver/`
2. Run `go test -v ./... -race` (race detection)
3. Cross-compile: `GOOS=windows GOARCH=amd64 go build -o dinoforge-resolver.exe`
4. Create PR (polyglot-build.yml triggers)

### Adding a Zig Optimization

1. Modify `src/Tools/AssetPipelineZig/src/root.zig`
2. Run `zig test src/root.zig`
3. Build: `zig build -O ReleaseFast`
4. Verify C# P/Invoke bindings
5. Create PR (polyglot-build.yml triggers)

### Debugging Game Bridge

1. Start MCP server: `python -m dinoforge_mcp.server --http --port 8765`
2. Launch game with BepInEx + DINOForge mod
3. Call MCP tools via Claude Code (`game_*` commands)
4. Check logs: `BepInEx/dinoforge_debug.log`

## Known Limitations & TODOs

- **Zig**: Build requires Zig 0.16.0+ (master branch), not stable 0.12.x
- **Rust PyO3**: Module not yet live-tested; fallback to C# PackCompiler works
- **Go**: Intentionally zero external dependencies (stdlib only)
- **Python tests**: Use mocks, not live game connections (for CI speed)
- **Mojo integration**: TODO — placeholder at `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py:15`
  - Target: Replace CLIP inference with Mojo kernels (~10x faster)
  - Blocker: Mojo stdlib stability (v0.5.0 today, needs v1.0+)
  - ETA: Late 2026 / Q1 2027

## Troubleshooting

### Rust artifact missing

- Check `Cargo.toml` has `crate-type = ["cdylib", "rlib"]`
- Verify PyO3 dependencies: `cargo tree | grep pyo3`
- Build locally: `cargo build --release`

### Go tests fail

- Compile locally: `go build main.go` → should succeed
- Check for external deps: `go mod tidy && go mod graph`
- No CGO required: `CGO_ENABLED=0 go test -v ./...`

### Zig build fails

- Verify Zig version: `zig version` (needs 0.16.0+)
- Build locally: `zig build -O ReleaseFast`
- Check for compile errors: `zig build-lib -O ReleaseFast src/root.zig`

### C# P/Invoke bindings fail at runtime

- Verify binary exists at expected path: `src/Bridge/Client/bin/Release/net8.0/`
- Check DLL name matches `[DllImport("...")]` attribute
- On Windows: Ensure DLLs are 64-bit (x86_64)
- On Linux: Ensure .so files are present (not just .dll)

### Python MCP server won't start

- Check dependencies: `pip list | grep fastmcp`
- Verify Python version: `python --version` (3.10+)
- Try isolated venv: `python -m venv venv && source venv/bin/activate && pip install -e .`
- Check for port conflicts: `netstat -an | grep 8765`

## References

- **Build System**: `Directory.Build.props` (MSBuild), `Taskfile.yml` (task runner)
- **Rust**: https://doc.rust-lang.org/cargo/ (Cargo), https://pyo3.rs/ (PyO3)
- **Go**: https://golang.org/doc/ (Go manual), https://github.com/actions/setup-go (GitHub Actions)
- **Zig**: https://ziglang.org/documentation/ (Zig docs), https://github.com/goto-bus-stop/setup-zig (GitHub Actions)
- **Python**: https://fastmcp.readthedocs.io/ (FastMCP), https://docs.pytest.org/ (pytest)
- **C#**: https://docs.microsoft.com/en-us/dotnet/csharp/ (.NET docs)

---

**Phase**: 3 (Polyglot Integration)
**Status**: ACTIVE
**Last Updated**: 2026-04-05
