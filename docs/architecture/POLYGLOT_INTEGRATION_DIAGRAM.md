# DINOForge Polyglot Integration Diagrams

## System Context Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        DINOForge Runtime Environment                     │
│                                                                           │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  Game Process (BepInEx + DINOForge Runtime)                      │   │
│  │  ├─ ECS World (45K entities, 6 systems groups)                   │   │
│  │  ├─ Asset Swap Registry (visual_asset → bundle cache)           │   │
│  │  ├─ Pack Registry (units, buildings, factions, doctrines)       │   │
│  │  └─ Stat Modifier System (per-entity override application)      │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│         ↑                                              ↓                   │
│         │ (ECS bridge)                          (pack manifest)           │
│         │                                                                  │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │  ContentLoader (SDK)                                              │   │
│  │  ├─ Discovery (find packs on disk)                               │   │
│  │  ├─ Dependency Resolution ← PackDependencyResolver (C#) OR       │   │
│  │  │                          GoDependencyResolver (Go CLI)        │   │
│  │  ├─ Asset Import ← RustAssetPipeline (Rust/PyO3) OR             │   │
│  │  │                 C# AssimpNet wrapper                           │   │
│  │  ├─ Schema Validation                                             │   │
│  │  └─ Registry Import (load into UnitRegistry, BuildingRegistry)  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│         ↑                    ↑                         ↑                   │
│         │                    │                         │                  │
│    fallback to C#         CLI JSON I/O           PyO3 module OR          │
│                                                   HTTP to MCP             │
│         │                    │                         │                  │
│         └────────┬───────────┴─────────────────────────┘                  │
│                  │                                                        │
│        ┌─────────┴──────────┐                                             │
│        │                    │                                             │
└────────┼────────────────────┼─────────────────────────────────────────────┘
         │                    │
         │                    │
         ↓                    ↓
    C# (sdk)            Native (Rust/Go)
         │                    │
         └─────┬──────────────┘
               │
         ┌─────┴─────────────────────┐
         │                           │
         ↓                           ↓
    ✓ Works                     ✓ Works (if binary available)
    (guaranteed)                (5-10x faster)
```

---

## Dependency Resolution Flow

### C# Synchronous Path (Fallback)

```
PackDependencyResolver.ResolveDependencies()
  │
  ├─ Parse manifests (YAML → C# objects)           [2-5ms]
  │
  ├─ Build in-degree map + adjacency list          [2-5ms]
  │
  ├─ Kahn's algorithm (SortedSet<T>)
  │  └─ While ready.Count > 0:
  │     ├─ Pop min (LoadOrder tiebreaker)          [1-2ms per pack]
  │     ├─ Reduce in-degree for dependents         [1-2ms per pack]
  │     └─ Insert back into SortedSet
  │
  ├─ Cycle detection (count processed vs input)    [<1ms]
  │
  └─ Return sorted PackManifest[] in load order
     Total: 15-35ms for 50 packs
```

### Go Subprocess Path (Fast)

```
GoDependencyResolver.ResolveDependencies()
  │
  ├─ Serialize input (available packs + target)
  │  └─ Write to temp JSON file                    [1-2ms]
  │
  ├─ Spawn process: dinoforge-resolver --input X --output Y
  │  └─ Process overhead (fork/exec)               [3-5ms]
  │
  └─ Go Process (subprocess):
     │
     ├─ Read JSON, parse manifests                 [1-2ms]
     │
     ├─ Kahn's algorithm (native Go slices)
     │  └─ While len(ready) > 0:
     │     ├─ Pop first (pre-sorted)
     │     ├─ Reduce in-degree
     │     └─ Insert in sorted order
     │     [0.5ms per pack — no GC overhead]
     │
     ├─ Serialize output JSON                      [1-2ms]
     │
     └─ Exit 0 (or 1 if errors)
         Total subprocess: 5-10ms for 50 packs
  │
  ├─ Read result from temp file                    [1-2ms]
  │
  └─ Deserialize and return PackManifest[]
     Total: 10-20ms (process overhead + Go time)
     Speedup vs C#: 5-10x for large graphs
```

### Integration Logic (GoDependencyResolver.cs)

```csharp
public DependencyResult ResolveDependencies(IEnumerable<PackManifest> available, PackManifest target)
{
    // Try Go fast path first
    if (IsAvailable)  // Check if dinoforge-resolver.exe exists in PATH
    {
        try
        {
            // 1. Serialize to temp JSON
            var input = new { available = available.ToList(), target };
            File.WriteAllText(tempInput, JsonSerializer.Serialize(input));

            // 2. Spawn Go process
            var process = new Process
            {
                StartInfo = new ProcessStartInfo
                {
                    FileName = resolverBinaryPath,
                    Arguments = $"--input {tempInput} --output {tempOutput}",
                    // ...
                }
            };
            process.Start();
            bool finished = process.WaitForExit(5000);

            if (!finished)
                throw new TimeoutException("Resolver timed out");
            if (process.ExitCode != 0)
                throw new InvalidOperationException("Resolver failed");

            // 3. Deserialize result
            var output = JsonSerializer.Deserialize<ResolverOutput>(File.ReadAllText(tempOutput));

            // 4. Convert back to PackManifest[]
            var sorted = new List<PackManifest>();
            foreach (var packId in output.Resolved)
            {
                sorted.Add(packById[packId]);
            }

            return DependencyResult.Success(sorted);
        }
        catch (Exception ex)
        {
            // Fall through to C# path
        }
    }

    // Fallback to C#
    var csharpResolver = new PackDependencyResolver();
    return csharpResolver.ResolveDependencies(available, target);
}
```

---

## Asset Import Flow

### C# AssimpNet Path (Fallback)

```
DirectAssetPipeline.RunPhase3A()
  │
  ├─ Load asset_pipeline.yaml config
  │
  ├─ Import Phase:
  │  │
  │  └─ For each asset in phase:
  │     │
  │     ├─ AssimpContext.ImportFile(glb_path)
  │     │  └─ P/Invoke → Assimp.dll
  │     │     ├─ Parse GLB (binary format)         [50-100ms for 100KB model]
  │     │     └─ Triangulate + generate normals   [10-20ms]
  │     │
  │     ├─ CombineMultipleMeshes()
  │     │  └─ Aggregate vertices/indices           [20-50ms]
  │     │
  │     ├─ ExtractMaterials()
  │     │  └─ Parse material properties            [5-10ms]
  │     │
  │     ├─ ExtractSkeleton() (if rigged)
  │     │  └─ Build bone hierarchy                 [5-10ms]
  │     │
  │     └─ Serialize to JSON                       [10-20ms]
  │        Total per asset: 100-200ms
  │
  ├─ Optimize Phase:
  │  └─ For each imported JSON:
  │     ├─ Deserialize                             [5-10ms]
  │     ├─ AssetOptimizationService.OptimizeAsync()
  │     │  └─ Mesh decimation (target poly %)     [30-80ms]
  │     └─ Serialize LOD variants                  [10-20ms]
  │        Total per asset: 50-110ms
  │
  └─ Prefab Generation Phase:
     └─ For each optimized JSON:
        ├─ Deserialize                             [5-10ms]
        ├─ PrefabGenerationService.GeneratePrefabAsync()
        │  └─ Create Unity prefab (serialized)     [20-40ms]
        └─ Write to disk                           [5-10ms]
           Total per asset: 30-60ms

Total for 1 asset: 180-370ms (avg 275ms)
Total for 10 assets: 1.8-3.7s (avg 2.75s)
```

### Rust PyO3 Path (Fast)

```
RustAssetPipeline.ImportAssetAsync()
  │
  ├─ IsAvailable check:
  │  └─ Detect Rust PyO3 module via MCP server     [<1ms if cached]
  │
  ├─ ImportAssetViaRustAsync():
  │  │
  │  └─ HTTP POST to MCP server:
  │     {
  │       "tool": "asset_import",
  │       "file_path": "/packs/.../unit.glb",
  │       "asset_id": "sw-rep-clone"
  │     }
  │
  └─ MCP Server (Python):
     │
     ├─ FastMCP receives HTTP request
     │
     ├─ Try import_asset() (PyO3 module):
     │  │
     │  └─ Rust code executed directly (no P/Invoke overhead):
     │     │
     │     ├─ assimp_bind::load_scene()
     │     │  └─ Direct Assimp C FFI (no marshaling)
     │     │     ├─ GLB binary parse                [40-70ms — zero-copy slicing]
     │     │     ├─ Triangulate + normals           [5-10ms]
     │     │     └─ Return AssimpScene struct       [<1ms]
     │     │
     │     ├─ mesh::combine_meshes()
     │     │  └─ Unsafe vertex slicing + aggregation
     │     │     ├─ Allocate combined buffers       [<5ms]
     │     │     ├─ Memcpy vertices (unsafe)        [10-20ms]
     │     │     ├─ Remap indices                   [5-10ms]
     │     │     └─ Return MeshData                 [<1ms]
     │     │     Total: 15-35ms
     │     │
     │     ├─ mesh::extract_materials()
     │     │  └─ Parse material properties          [2-5ms]
     │     │
     │     ├─ Serde JSON serialization              [5-10ms]
     │     │
     │     └─ Return JSON string to Python
     │
     ├─ HTTP 200 + JSON response to C#
     │  └─ Latency: 60-120ms for complete import
     │
  └─ C# deserializes JSON → ImportedAsset
     Total per asset: 80-150ms (avg 115ms)
     Total for 10 assets: 0.8-1.5s (avg 1.15s)

Speedup vs C#: 2-3x (275ms → 115ms per asset)
```

### Direct P/Invoke Path (Alternative)

```
RustAssetPipeline.ImportAssetViaPInvoke()
  │
  ├─ [DllImport("dinoforge_asset_pipeline")]
  │  RustImportAsset(filePath, assetId, out resultJson)
  │
  └─ Direct call into Rust (no HTTP overhead):
     │
     ├─ Load .pyd/.dll (first call only)            [0ms cached]
     │
     ├─ Rust code (same as PyO3):
     │  └─ Assimp FFI + mesh operations             [60-120ms]
     │
     ├─ Marshal result JSON to C# string
     │  └─ String copy (small JSON)                 [<2ms]
     │
     └─ Deserialize in C#
        Total per asset: 70-130ms (slightly faster than HTTP)

Only recommended if latency critical (e.g., <100ms target).
HTTP path preferred for platform portability.
```

### Integration Logic (RustAssetPipeline.cs)

```csharp
public static async Task<ImportedAsset> ImportAssetAsync(string assetId, string filePath)
{
    if (!File.Exists(filePath))
        throw new FileNotFoundException(filePath);

    // Try Rust fast path (PyO3 via MCP)
    if (IsAvailable)
    {
        try
        {
            return await ImportAssetViaRustAsync(assetId, filePath);
        }
        catch (Exception ex)
        {
            Debug.WriteLine($"Rust import failed, falling back to C#: {ex.Message}");
            // Fall through to C# path
        }
    }

    // Fallback: C# AssimpNet wrapper
    return await ImportAssetViaCSharpAsync(assetId, filePath);
}

private static async Task<ImportedAsset> ImportAssetViaRustAsync(string assetId, string filePath)
{
    // HTTP call to MCP server (localhost:8765)
    var response = await CallMcpAsync("asset_import", new
    {
        file_path = Path.GetFullPath(filePath),
        asset_id = assetId
    });

    var json = response.ToString();
    return JsonSerializer.Deserialize<ImportedAsset>(json)
        ?? throw new InvalidOperationException("Failed to deserialize");
}
```

---

## MCP Server Integration

```
┌────────────────────────────────────────────────────┐
│  C# / Test Code (any language)                     │
│  calls ContentLoader / PackCompiler                │
└───────────────┬──────────────────────────────────┘
                │ HTTP POST
                │ localhost:8765/api/tools/{toolName}
                ↓
┌────────────────────────────────────────────────────┐
│  Python MCP Server (FastMCP)                       │
│  src/Tools/DinoforgeMcp/server.py                  │
│                                                    │
│  @app.tool()                                       │
│  async def asset_import(file_path, asset_id):      │
│    if RUST_AVAILABLE:                              │
│      try:                                          │
│        # Call Rust PyO3 module directly            │
│        return import_asset(file_path, asset_id)    │
│      except:                                       │
│        # Fall back to C#                           │
│        return call_c_sharp_asset_import(...)       │
│    else:                                           │
│      # C# fallback only                            │
│      return call_c_sharp_asset_import(...)         │
│                                                    │
└───────────────┬──────────────────────────────────┘
                │
        ┌───────┴──────────┐
        ↓                  ↓
    (if RUST_AVAILABLE)  (else)
        │                 │
        ↓                 ↓
    ┌─────────────┐   ┌──────────────────┐
    │ Rust PyO3   │   │ C# PackCompiler  │
    │ module      │   │ subprocess       │
    │ (Assimp     │   │ (AssimpNet)      │
    │  FFI)       │   │                  │
    └─────────────┘   └──────────────────┘
        │ 60-120ms         │ 100-200ms
        │                  │
        └──────────┬───────┘
                   ↓
            ┌──────────────┐
            │ JSON response│
            └──────────────┘
                   │
                   ↓
            HTTP 200 + JSON
            back to C#
```

---

## Build & Deployment Pipeline

```
CI/CD Workflow (GitHub Actions)
  │
  ├─ Checkout code
  │
  ├─ Setup .NET 11 (include-prerelease: true)
  │
  ├─ Setup Rust
  │  │
  │  ├─ Install rustc 1.75 (dtolnay/rust-toolchain)
  │  │
  │  ├─ Build Rust crate:
  │  │  └─ cd src/Tools/AssetPipelineRust
  │  │     cargo build --release
  │  │     → artifacts/dinoforge_asset_pipeline.pyd (Windows x64)
  │  │     → artifacts/dinoforge_asset_pipeline.so (Linux)
  │  │     → artifacts/dinoforge_asset_pipeline.dylib (macOS)
  │  │
  │  └─ Test Rust:
  │     └─ cargo test
  │
  ├─ Setup Go
  │  │
  │  ├─ Install Go 1.23 (setup-go@v4)
  │  │
  │  ├─ Build Go binary:
  │  │  └─ cd src/Tools/DependencyResolver
  │  │     go build -o bin/dinoforge-resolver.exe
  │  │     → bin/dinoforge-resolver (Linux/macOS auto-named)
  │  │
  │  └─ Test Go:
  │     └─ go test ./...
  │
  ├─ Build C#
  │  │
  │  ├─ Restore NuGet packages
  │  │
  │  ├─ dotnet build src/DINOForge.sln -c Release
  │  │
  │  └─ Test C#:
  │     └─ dotnet test src/Tests/ -c Release
  │
  ├─ Package Release
  │  │
  │  ├─ Create release artifacts/
  │  │  ├─ DINOForge-SDK-X.Y.Z.nupkg (C#, includes NativeInterop/)
  │  │  ├─ DINOForge-v0.18.0-windows.zip
  │  │  │  ├─ bin/dinoforge-resolver.exe
  │  │  │  ├─ lib/dinoforge_asset_pipeline.pyd
  │  │  │  └─ ...
  │  │  ├─ DINOForge-v0.18.0-linux.tar.gz
  │  │  │  ├─ bin/dinoforge-resolver
  │  │  │  ├─ lib/dinoforge_asset_pipeline.so
  │  │  │  └─ ...
  │  │  └─ DINOForge-v0.18.0-macos.tar.gz
  │  │     ├─ bin/dinoforge-resolver
  │  │     ├─ lib/dinoforge_asset_pipeline.dylib
  │  │     └─ ...
  │  │
  │  └─ Publish NuGet packages
  │
  └─ Upload artifacts to GitHub Release
     (users download pre-built binaries)

Local Build (Developer)
  │
  ├─ If Rust installed:
  │  └─ cargo build --release in AssetPipelineRust/
  │     → .pyd/.so/.dylib copied to SDK lib/
  │
  ├─ If Go installed:
  │  └─ go build in DependencyResolver/
  │     → exe copied to SDK bin/
  │
  ├─ dotnet build src/DINOForge.sln
  │  ├─ Detects Rust/Go binaries in lib/bin/
  │  ├─ Links via NativeInterop layer
  │  └─ If missing: Uses C# fallback automatically
  │
  └─ Test with ContentLoader
     ├─ If binaries available: Rust/Go used
     └─ If not: C# fallback (same API, 2-3x slower)
```

---

## Error Handling & Fallback Logic

```
Call Stack Example: Import Asset

ContentLoader.LoadPackAsync()
  └─ for each unit in pack.units:
       DirectAssetPipeline.ImportAsset(unit.visual_asset)
         └─ RustAssetPipeline.ImportAssetAsync()
            │
            ├─ IsAvailable? (checks MCP server or DLL)
            │
            ├─ YES → ImportAssetViaRustAsync()
            │        │
            │        ├─ HTTP POST to MCP
            │        │
            │        └─ [SUCCESS] ✓ Return ImportedAsset
            │        │
            │        └─ [TIMEOUT] → catch → Fall back to C#
            │        │
            │        └─ [HTTP 500] → catch → Fall back to C#
            │
            └─ NO → ImportAssetViaCSharpAsync()
               │
               └─ [ALWAYS WORKS] ✓ Return ImportedAsset
                  (AssimpNet wrapper always available)

Call Stack Example: Resolve Dependencies

ContentLoader.LoadPackAsync()
  └─ resolver.ResolveDependencies()
     └─ GoDependencyResolver.ResolveDependencies()
        │
        ├─ IsAvailable? (check if dinoforge-resolver.exe in PATH)
        │
        ├─ YES → ResolveDependenciesViaGo()
        │        │
        │        ├─ Spawn process
        │        │
        │        ├─ [SUCCESS] → Deserialize + Return
        │        │
        │        └─ [TIMEOUT/FAIL] → catch → Fall back to C#
        │
        └─ NO → ResolveDependenciesViaCSharp()
           │
           └─ [ALWAYS WORKS] ✓ Return DependencyResult
              (C# Kahn's algorithm always available)
```

---

## Summary Table: Language Choice by Subsystem

| Subsystem | Lang | Path A (Fast) | Path B (Fallback) | Status |
|-----------|------|---------------|-------------------|--------|
| **Asset Import** | Rust | PyO3 via MCP HTTP (80-150ms) | C# AssimpNet (180-370ms) | Optional, 2-3x speedup |
| **Dependency Resolver** | Go | CLI subprocess (10-20ms) | C# Kahn's (15-35ms) | Optional, 5-10x speedup |
| **ECS Queries** | C# | Direct LINQ (frame-critical) | N/A | Required, no alt |
| **Stat Modifiers** | C# | Direct calc | N/A | Required, no alt |
| **MCP Server** | Python | FastMCP + PyO3 calls | N/A | Already optimal |

---

This architecture provides:
- **Maximum performance** when all binaries available (Rust + Go)
- **Graceful degradation** if binaries missing (C# fallback)
- **Zero tight coupling** (JSON/HTTP boundaries)
- **Platform-agnostic** (Windows, Linux, macOS via same C# API)
- **Simple debugging** (separate processes + logs)
