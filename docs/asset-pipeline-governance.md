# Asset Pipeline Governance (v0.7.0+)

All asset operations (3D models, textures, VFX) MUST go through **PackCompiler commands** — never fragmented/legacy tools.

## Unified Asset Workflows (PackCompiler)

```bash
dotnet run --project src/Tools/PackCompiler -- assets import <pack>
dotnet run --project src/Tools/PackCompiler -- assets validate <pack>
dotnet run --project src/Tools/PackCompiler -- assets optimize <pack>
dotnet run --project src/Tools/PackCompiler -- assets generate <pack>
dotnet run --project src/Tools/PackCompiler -- assets build <pack>
dotnet run --project src/Tools/PackCompiler -- sync download <pack> --phase <version>
dotnet run --project src/Tools/PackCompiler -- vfx generate <pack>   # wraps VFXPrefabGenerator
```

## Asset Configuration: asset_pipeline.yaml

Every pack with assets MUST define `asset_pipeline.yaml` with: model sources (GLB/FBX paths), LOD targets (polycount %, screen thresholds), material definitions (faction colors/emission), Addressables keys, and definition updates (inject `visual_asset` references). Schema: `schemas/asset_pipeline.schema.json`.

## Mandatory Asset Workflow Steps (in order)

1. **Define** — Create/update `asset_pipeline.yaml` in pack root
2. **Download** — `sync download <pack>`
3. **Import** — `assets import <pack>`
4. **Validate** — `assets validate <pack>`
5. **Optimize** — `assets optimize <pack>` (generates LOD)
6. **Generate** — `assets generate <pack>` (creates prefabs)
7. **Verify** — `assets build <pack>` (full pipeline + tests)
8. **Commit** — Git commit all artifacts + updated definitions

**Agents MUST NOT**: manually edit game definitions when assets change; skip validation/optimization; create ad-hoc asset directories outside `packs/<pack>/assets/`; hardcode polycount targets or LOD percentages in C#; use separate/legacy tools.

## Asset Services (PackCompiler)

Core services in `src/Tools/PackCompiler/Services/`.

> **STATUS (iter-144 audit a82aaf707e8907d1a):** Aspirational test coverage claim was false. `AssetPipelineTests.cs` does NOT exist. Services HAVE real implementations (no `NotImplementedException`) but C# behavior is unverified. Track as #589 — write the 20+ tests OR accept FsCheck-only coverage.

| Service | Responsibility | Tests |
|---------|-----------------|-------|
| `AssetImportService` | GLB/FBX → JSON (AssimpNet) | FsCheck only (`src/Tests/ParameterizedTests/AssetPipelineFsCheckProperties.cs`) |
| `AssetOptimizationService` | Mesh decimation → LOD variants | FsCheck only |
| `PrefabGenerationService` | JSON → .prefab (serialized) | FsCheck only |
| `AddressablesService` | YAML → catalog entries | FsCheck only |
| `DefinitionUpdateService` | Inject visual_asset into YAML | FsCheck only |

## Extension Pattern

Custom processors/validators register in PackCompiler DI setup; implementations MUST inherit `IAssetProcessor`, `IAssetValidator`, or `IAssetExporter`:

```csharp
public static IServiceCollection AddCustomAssetProcessors(this IServiceCollection services)
{
    services.AddAssetProcessor<CustomLightsaberGlowProcessor>();
    services.AddAssetValidator<StarWarsColorValidator>();
    services.AddAssetExporter<AlternativeFormatExporter>();
    return services;
}
```

## Testing Requirements for Assets

New asset features MUST include: unit tests per service; integration tests for full pipeline; regression tests for known assets; performance tests (import < 5s/model, full pipeline < 5min for 9 models); schema validation tests. Tests live in `src/Tests/AssetPipelineTests.cs` (see #589 status above).

## Documentation Requirements

Agents changing asset workflows MUST update: `ASSET_PIPELINE_CLI.md` (command reference), `asset_pipeline.schema.json` (config schema), this doc (governance), inline XML docs in PackCompiler services, and test cases documenting new behavior.

---

## Asset Bundle Creation

### Unity Version Requirement
Asset bundles for DINO **must be built with Unity 2021.3.45f2**. Bundles from other versions fail to load silently at runtime.

### Bundle → Pack Registration Flow
```
1. source.glb/fbx
2. → blender normalize_asset.py → normalized.glb (polycount reduction, material cleanup)
3. → blender stylize_asset.py → stylized.glb (faction palette applied)
4. → Unity 2021.3: import stylized.glb, build AssetBundle → <asset-id> (no extension)
5. → packs/<pack-id>/assets/bundles/<asset-id>   (bundle file)
6. → unit/building YAML: visual_asset: <asset-id>
7. → dotnet build -p:DeployToGame=true
```

### Bundle Naming Convention
- Bundle filename = `visual_asset` key = Addressable key used by AssetSwapRegistry
- Example: `sw-rep-clone-trooper` for file `packs/warfare-starwars/assets/bundles/sw-rep-clone-trooper`
- The asset name **inside** the bundle is the Unity prefab name (e.g. `sw-rep-clone-trooper.prefab`)
- AssetSwapSystem uses `bundle.LoadAllAssets()` fallback to handle name mismatches

### Faction Palettes (hardcoded in `AssetctlPipeline.BuildFactionPalette`)
| Faction | Primary | Secondary | Roughness | Metallic |
|---------|---------|-----------|-----------|---------|
| republic | `#F5F5F5` | `#1A3A6B` | 0.3 | 0.1 |
| cis | `#C8A87A` | `#5C3D1E` | 0.7 | 0.2 |
| neutral | `#888888` | — | 0.5 | 0.0 |
