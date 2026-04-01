# Phase 3D Integration Checklist

**Date**: 2026-03-31
**Status**: CI/CD WORKFLOW CREATED
**Next**: C# Bridge Integration

## What Was Built

✓ **Workflow File**: `.github/workflows/polyglot-build.yml` (148 lines)
- Job 1: Build Rust asset pipeline (Ubuntu, release mode)
- Job 2: Build Go resolver (Ubuntu, cross-compile Windows)
- Job 3: Verify artifacts (download & validate)
- Triggers: Push to main, all PRs, path filters on polyglot source

## Artifact Pipeline

When workflow runs, it produces these artifacts (30-day retention):

```
GitHub Actions Artifact Store
├── rust-asset-pipeline
│   └── libdinoforge_asset_pipeline.so (Linux x64)
├── go-resolver-linux
│   └── dinoforge-resolver (Linux x64)
└── go-resolver-windows
    └── dinoforge-resolver.exe (Windows x64)
```

## C# Integration Tasks (Next Phase)

### Task A: Download Artifacts in C# Build
**File**: `.github/workflows/ci.yml` (existing C# workflow)

```yaml
- name: Download polyglot artifacts
  uses: actions/download-artifact@v4.1.8
  with:
    path: polyglot-binaries/

- name: Prepare Rust library
  run: |
    cp polyglot-binaries/rust-asset-pipeline/libdinoforge_asset_pipeline.so \
       src/Bridge/Client/bin/Release/net8.0/libdinoforge_asset_pipeline.so

- name: Prepare Go resolver
  run: |
    mkdir -p src/Bridge/Client/bin/Release/net8.0/bin
    cp polyglot-binaries/go-resolver-linux/dinoforge-resolver \
       src/Bridge/Client/bin/Release/net8.0/bin/
    cp polyglot-binaries/go-resolver-windows/dinoforge-resolver.exe \
       src/Bridge/Client/bin/Release/net8.0/bin/
```

### Task B: P/Invoke Bindings (DINOForge.Bridge.Client)
**File**: `src/Bridge/Client/PolyglotInterop.cs` (create new)

```csharp
using System.Runtime.InteropServices;

namespace DINOForge.Bridge.Client.Polyglot;

/// <summary>
/// Rust asset pipeline P/Invoke interface
/// </summary>
public static class AssetPipelineInterop
{
    private const string LibraryName = "libdinoforge_asset_pipeline";

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int asset_import_mesh(string glb_path, string output_json);

    [DllImport(LibraryName, CallingConvention = CallingConvention.Cdecl)]
    public static extern int asset_generate_lods(string mesh_json, int target_count);
}

/// <summary>
/// Go dependency resolver subprocess wrapper
/// </summary>
public static class ResolverInterop
{
    public static async Task<(string[] resolved, string[] errors)> ResolveDependencies(
        string inputJson,
        CancellationToken ct = default)
    {
        var processInfo = new ProcessStartInfo
        {
            FileName = "bin/dinoforge-resolver",
            ArgumentList =
            {
                "--input", Path.GetTempFileName(),
                "--output", Path.GetTempFileName()
            },
            RedirectStandardOutput = true,
            UseShellExecute = false
        };

        // Write input, launch resolver, read output
        // Return parsed (resolved, errors) tuple
    }
}
```

### Task C: Integration Tests
**File**: `src/Tests/Integration/PolyglotInteropTests.cs` (create new)

```csharp
[Trait("Category", "Polyglot")]
public class AssetPipelineTests
{
    [Fact]
    public void ImportMesh_WithValidGLB_ReturnsSuccess()
    {
        // Arrange: Load test.glb
        // Act: Call AssetPipelineInterop.asset_import_mesh()
        // Assert: Output JSON is valid, contains mesh data
    }

    [Fact]
    public void GenerateLODs_WithMeshJSON_GeneratesVariants()
    {
        // Arrange: Create mesh JSON fixture
        // Act: Call AssetPipelineInterop.asset_generate_lods()
        // Assert: LOD variants present in output
    }
}

[Trait("Category", "Polyglot")]
public class ResolverTests
{
    [Fact]
    public async Task ResolveDependencies_WithLinearDeps_ReturnsCorrectOrder()
    {
        // Arrange: Create packs: A -> B -> C
        // Act: Call ResolverInterop.ResolveDependencies()
        // Assert: resolved = [C, B, A]
    }

    [Fact]
    public async Task ResolveDependencies_WithCycle_ReturnsError()
    {
        // Arrange: Create packs: A -> B -> A (cycle)
        // Act: Call ResolverInterop.ResolveDependencies()
        // Assert: errors contains "Circular dependency"
    }
}
```

### Task D: ContentLoader Integration
**File**: `src/SDK/ContentLoader.cs` (extend existing)

```csharp
public class ContentLoader
{
    private readonly AssetPipelineService _assetPipeline;
    private readonly DependencyResolverService _resolver;

    public async Task<ContentPackage[]> LoadAndResolveAsync(string packDir)
    {
        // 1. Scan packs directory
        // 2. Call _resolver.ResolveAsync() to get load order
        // 3. Call _assetPipeline.ImportAsync() for each pack's assets
        // 4. Return ordered packages with resolved dependencies
    }
}
```

### Task E: Documentation
**Files to Update**:
- `docs/POLYGLOT_INTEGRATION.md` - Full integration guide
- `docs/ARCHITECTURE.md` - Update diagrams (add Rust/Go boxes)
- `src/Bridge/Client/README.md` - P/Invoke examples

## Timeline

| Task | Est. Time | Owner | Status |
|------|-----------|-------|--------|
| A: Artifact download | 1 hr | - | TODO |
| B: P/Invoke bindings | 2 hrs | - | TODO |
| C: Integration tests | 3 hrs | - | TODO |
| D: ContentLoader extend | 2 hrs | - | TODO |
| E: Documentation | 1 hr | - | TODO |
| **Total** | **~9 hrs** | - | **PENDING** |

## Build Order (CI Dependencies)

1. **polyglot-build.yml** runs first (Rust + Go)
   - Produces 3 artifacts
   - Takes ~5-8 minutes total

2. **ci.yml** runs next (C# build)
   - Downloads polyglot artifacts
   - Compiles with P/Invoke bindings
   - Runs integration tests
   - Takes ~10-15 minutes

## Testing Strategy

### Local Development
```bash
# Build Rust locally
cd src/Tools/AssetPipelineRust
cargo build --release

# Build Go locally
cd src/Tools/DependencyResolver
go build -o bin/dinoforge-resolver main.go

# Link binaries to C# project
cp target/release/libdinoforge_asset_pipeline.so ../../../Bridge/Client/bin/Release/net8.0/
cp bin/dinoforge-resolver ../../../Bridge/Client/bin/Release/net8.0/bin/

# Run C# tests (P/Invoke tests should pass)
dotnet test src/Tests/Integration/PolyglotInteropTests.cs
```

### CI Testing
- Workflow runs automatically on push/PR
- Artifacts downloaded and linked
- P/Invoke tests run as part of `dotnet test`
- Verification job ensures binaries present

## Success Criteria

- [ ] Artifacts downloaded successfully in C# build
- [ ] P/Invoke calls work (no DLL load errors)
- [ ] Asset import test passes
- [ ] Dependency resolver test passes
- [ ] Integration test with real packs passes
- [ ] All 3 binaries present in release build output

## Known Issues / TODOs

1. **Platform Detection**: P/Invoke needs to detect Linux vs. Windows and load correct binary
   - Solution: Platform-specific paths or use `RuntimeInformation.IsOSPlatform()`

2. **Process Isolation**: Go resolver spawns subprocess; ensure proper cleanup
   - Solution: Use `ProcessStartInfo` with proper IDisposable handling

3. **Error Handling**: Both libraries should return structured error codes
   - Rust: `Result<T, E>` exposed as return codes
   - Go: Already returns JSON error field

4. **Versioning**: Ensure C# can handle polyglot versions mismatch
   - Solution: Add version check at startup

## References

- Workflow: `.github/workflows/polyglot-build.yml`
- Rust Library: `src/Tools/AssetPipelineRust/Cargo.toml`
- Go Resolver: `src/Tools/DependencyResolver/main.go`
- C# Bridge: `src/Bridge/Client/DINOForge.Bridge.Client.csproj`
- Documentation: `docs/sessions/polyglot-build-workflow.md`

---

**Phase 3D Status**: CI/CD COMPLETE, AWAITING C# INTEGRATION
