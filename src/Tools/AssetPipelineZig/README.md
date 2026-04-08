# Zig Asset Pipeline

High-performance mesh decimation and spatial indexing for DINOForge.

Built with Zig 0.16.0-dev for native compilation to DLL/SO libraries.

## Building

### Compile as shared library

```bash
zig build-lib -O ReleaseFast -dynamic src/root.zig
```

Output: `root.dll` (Windows) or `root.so` (Linux)

Rename to: `dinoforge_asset_pipeline_zig.dll`

### Run tests

```bash
zig test src/root.zig
```

Expected output: All 7 tests pass
- Vec3 operations
- AABB contains point
- AABB intersection
- AABB center and size
- BVH query
- Mesh decimator init
- Mesh decimator decimate

## Features

### LOD Mesh Decimation

Garfield-Heckbert algorithm for reducing polygon counts while preserving mesh quality.

- Progressive reduction from 0-100% target polycount
- Maintains spatial coherence

### Spatial Indexing (AABB BVH)

Axis-Aligned Bounding Box Bounding Volume Hierarchy for fast spatial queries.

- Point containment tests
- AABB intersection detection
- Efficient range queries

## Integration

P/Invoke bindings: `src/SDK/NativeInterop/ZigAssetPipeline.cs`

Call native functions from C# at runtime:

```csharp
[DllImport("dinoforge_asset_pipeline_zig.dll")]
private static extern void DecimateMultiple(
    IntPtr meshes, 
    int count, 
    float targetPolycount);
```

## Testing

All 7 tests pass locally:

```bash
zig test src/root.zig
```

Tests cover:
- Vec3 vector operations
- AABB containment and intersection
- BVH spatial indexing
- Mesh decimation

## Compatibility

- **Zig version**: 0.16.0-dev.3132+fd2718f82 (master)
- **Target platforms**: Windows (x86_64), Linux (x86_64)
- **CI**: Automated builds on both platforms via GitHub Actions

## Development

Project structure:
```
AssetPipelineZig/
  src/
    root.zig      - Main module (Vec3, AABB, BVH, MeshDecimator)
  build.zig       - Build manifest
  build.zig.zon   - Package manifest
  README.md       - This file
```

### Adding new features

1. Edit `src/root.zig`
2. Add tests before implementation
3. Run `zig test src/root.zig` to verify
4. Update CI if adding new platforms

## License

Same as DINOForge (see parent project LICENSE)
