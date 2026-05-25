# DINOForge Pack Dependency Resolver

Fast topological sort for pack dependency resolution using Kahn's algorithm.

## Building

Requirements: Go 1.21+

```bash
# Build binary
go build -o dinoforge-resolver .

# Run tests
go test -v ./...
go test -v -race ./...  # Race condition detection
```

## Usage

**Input**: JSON manifest with pack dependencies

```json
{
  "packs": [
    {"id": "pack-a", "depends_on": ["pack-b"]},
    {"id": "pack-b", "depends_on": []},
    {"id": "pack-c", "depends_on": ["pack-a"]}
  ]
}
```

**Execution**:

```bash
./dinoforge-resolver --input manifest.json --output resolved.json
```

**Output**: Ordered list of packs (dependency order)

```json
{"sorted": ["pack-b", "pack-a", "pack-c"]}
```

## Features

- **Cycle Detection**: Returns error if circular dependencies found
- **Topological Sort**: Uses Kahn's algorithm (O(V+E) complexity)
- **Zero Dependencies**: Only stdlib (encoding/json, flag, os, log, fmt)
- **Cross-Platform**: Builds on Windows, macOS, Linux

## Integration

C# Interop: `src/SDK/NativeInterop/GoDependencyResolver.cs`
- Spawns resolver as subprocess
- Passes manifests via JSON temp files
- Falls back to C# implementation if binary unavailable

## Testing

```bash
go test -v ./...        # Unit tests
go test -race ./...     # Race conditions
go test -bench . ./...  # Benchmarks
```

## CI/CD

GitHub Actions workflow: `.github/workflows/polyglot-build.yml`
- Runs on: Linux (GitHub Actions runners)
- Setup: actions/setup-go@v4
- Build: `CGO_ENABLED=0 go build -ldflags="-s -w"` (static, stripped)
- Cross-compile: GOOS=windows GOARCH=amd64 for Windows binary
- Test: `go test -v ./...`
- Artifacts: Uploaded for both Linux and Windows (`go-resolver-linux`, `go-resolver-windows`)
- Retention: 30 days

## Troubleshooting

**Build fails with "undefined reference"**:
- Ensure `CGO_ENABLED=0` is set (static linking, no C dependencies)
- Go modules must be initialized: `go mod init` (if missing)

**Tests fail with race conditions**:
- Run `go test -race ./...` to detect data races
- Resolver is designed for single-threaded use; use goroutine-safe wrappers in C# layer

**Cross-compilation issues**:
- Verify Go version: `go version` (1.21+)
- Try building locally first: `go build .`
- Check GOOS/GOARCH targets: only windows/linux amd64 supported

## References

- Go Toolchain: https://github.com/actions/setup-go
- Go Manual: https://golang.org/doc/
- Kahn's Algorithm: https://en.wikipedia.org/wiki/Topological_sorting
