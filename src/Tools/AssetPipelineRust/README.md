# Rust Asset Pipeline (PyO3)

High-performance asset import and LOD generation for DINOForge via Python FFI.

## Building

Requirements: Rust 1.70+, Python 3.10+, Assimp

```bash
# Build release binary
cargo build --release

# Build Python extension module (.pyd on Windows, .so on Linux)
maturin develop --release

# Run tests
cargo test --release
```

## Output

- **Rust library**: `target/release/libdinoforge_asset_pipeline.a` (static)
- **Rust dynamic**: `target/release/dinoforge_asset_pipeline.dll` / `.so`
- **Python extension**: `dinoforge_asset_pipeline.pyd` (Windows) or `.so` (Linux)

## Features

- **Import Assets**: Parse GLB/FBX files (via Assimp) → JSON mesh data
- **Optimize Mesh**: LOD generation via Garfield-Heckbert decimation
- **Parallel Processing**: Rayon-based SIMD for multi-core mesh operations

## Integration

C# P/Invoke bindings: `src/SDK/NativeInterop/RustAssetPipeline.cs`

Python MCP server: `src/Tools/DinoforgeMcp/dinoforge_mcp/server.py`
- Fallback mechanism: If PyO3 module unavailable, uses C# PackCompiler

## Testing

```bash
cargo test --all --release
cargo bench  # Performance benchmarks
```

## Performance Targets

- Import GLB (1M polys): < 500ms
- LOD decimation (50%): < 200ms
- Parallel batch (8 models): < 5s total

## CI/CD

GitHub Actions workflow: `.github/workflows/polyglot-build.yml`
- Runs on: Linux (GitHub Actions runners)
- Setup: dtolnay/rust-toolchain@stable
- Commands: `cargo build --release` → `cargo test --release`
- Artifacts: `rust-asset-pipeline` uploaded (libdinoforge_asset_pipeline.so)
- Retention: 30 days

## Troubleshooting

**Cargo.toml compilation**:
- Ensure `crate-type = ["cdylib", "rlib"]` in Cargo.toml for both static and dynamic libs
- PyO3 dependencies must be pinned to compatible version

**Missing dependencies**:
- Install Assimp: `apt-get install libassimp-dev` (Linux) or `brew install assimp` (macOS)
- Verify Python headers: `python -m pip install pydantic`

**PyO3 module not loading**:
- Run `maturin develop --release` to rebuild Python extension
- Check Python version compatibility (3.10+)

## References

- Rust Toolchain: https://github.com/dtolnay/rust-toolchain
- Cargo: https://doc.rust-lang.org/cargo/
- PyO3 Docs: https://pyo3.rs/
- Maturin: https://www.maturin.rs/
