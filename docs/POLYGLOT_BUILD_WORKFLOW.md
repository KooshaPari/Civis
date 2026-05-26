# Polyglot Build Workflow

GitHub Actions workflow for building and testing Rust and Go binaries as part of DINOForge Phase 3D polyglot integration.

## Quick Reference

**File**: `.github/workflows/polyglot-build.yml`
**Status**: Active (Phase 3D)
**Trigger**: Push to `main` or PRs; filters on `src/Tools/AssetPipelineRust/**`, `src/Tools/DependencyResolver/**`
**Duration**: ~5-8 minutes

## Jobs

### 1. build-rust
Builds the Rust asset pipeline library for Linux.

```bash
cd src/Tools/AssetPipelineRust
cargo build --release
cargo test --release
```

**Artifact**: `rust-asset-pipeline` → `libdinoforge_asset_pipeline.so`

### 2. build-go
Builds the Go dependency resolver for both Linux and Windows.

```bash
cd src/Tools/DependencyResolver
# Linux
CGO_ENABLED=0 go build -ldflags="-s -w" -o bin/dinoforge-resolver main.go

# Windows (cross-compile from Linux)
GOOS=windows GOARCH=amd64 CGO_ENABLED=0 go build -ldflags="-s -w" -o bin/dinoforge-resolver.exe main.go

go test -v ./...
```

**Artifacts**:
- `go-resolver-linux` → `dinoforge-resolver`
- `go-resolver-windows` → `dinoforge-resolver.exe`

### 3. verify-artifacts
Downloads and validates all artifacts after both build jobs complete.

- Checks binary existence
- Reports file sizes
- Ensures artifacts ready for C# integration

## Artifacts

All artifacts retain for 30 days and are automatically downloadable from GitHub Actions.

| Name | Binary | Platform | Size (typical) |
|------|--------|----------|---|
| `rust-asset-pipeline` | libdinoforge_asset_pipeline.so | Linux x64 | 2-5 MB |
| `go-resolver-linux` | dinoforge-resolver | Linux x64 | 2-3 MB (stripped) |
| `go-resolver-windows` | dinoforge-resolver.exe | Windows x64 | 2-3 MB (stripped) |

## Local Build

### Rust (Linux/macOS/Windows)
```bash
cd src/Tools/AssetPipelineRust
rustup install stable
cargo build --release
cargo test --release
```

### Go (any platform)
```bash
cd src/Tools/DependencyResolver
go build -o bin/dinoforge-resolver main.go
go test -v ./...

# Cross-compile for Windows
GOOS=windows GOARCH=amd64 go build -o bin/dinoforge-resolver.exe main.go
```

## Next: C# Integration

The C# build (`ci.yml`) should:

1. Download polyglot artifacts from this workflow
2. Link binaries to `src/Bridge/Client/bin/Release/net8.0/`
3. Use P/Invoke to call Rust FFI and spawn Go subprocess
4. Run integration tests (`PolyglotInteropTests.cs`)

See: `docs/sessions/phase-3d-integration-checklist.md`

## Security

- All action references pinned to commit hashes
- Minimal permissions (`contents: read` only)
- No hardcoded secrets
- Build commands hardcoded (no untrusted interpolation)

## Troubleshooting

**Rust artifact missing**:
- Check `src/Tools/AssetPipelineRust/Cargo.toml` has `crate-type = ["cdylib", "rlib"]`
- Verify PyO3 dependencies installed

**Go tests fail**:
- Check `src/Tools/DependencyResolver/main.go` compiles locally: `go build main.go`
- No external dependencies (stdlib only)

**Artifacts not downloadable**:
- Check workflow "Summary" tab in GitHub Actions
- Artifacts expire after 30 days

## References

- Rust Toolchain: https://github.com/dtolnay/rust-toolchain
- Go Setup: https://github.com/actions/setup-go
- Artifact Upload/Download: https://github.com/actions/upload-artifact
- Workflow Documentation: `.github/workflows/polyglot-build.yml`

---

**Phase**: 3D (Polyglot Integration)
**Status**: ACTIVE
**Created**: 2026-03-31
