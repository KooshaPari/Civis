# Asset Pipeline Architecture - Long-Term Extensible Solution

**Date**: 2026-03-13
**Version**: 1.0 Design
**Scope**: v0.7.0 → v1.0.0+ (all future content packs)
**Goal**: Create reusable, testable asset import/processing infrastructure

---

## Vision

Instead of manual Unity Editor work, build a **declarative asset pipeline** that:

1. **Is CLI-based** (batch mode, agent-friendly, scriptable)
2. **Supports multiple formats** (GLB, FBX, OBJ, PNG, JSON)
3. **Chains processors** (import → validate → optimize → generate → build)
4. **Is testable** (unit tests for each step, reproducible builds)
5. **Scales to v1.0.0** (100+ models across 5+ packs)
6. **Integrates with PackCompiler** (unified `dinoforge` command)
7. **Is extensible** (plugin architecture for custom processors)

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Asset Pipeline CLI Tool                  │
│  src/Tools/AssetPipeline/                                   │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  Commands:                                                   │
│  ├─ import       Import GLB/FBX → intermediate format       │
│  ├─ validate     Check format, polycount, materials          │
│  ├─ optimize     Generate LOD variants, compress textures    │
│  ├─ generate     Create prefabs, addressables entries        │
│  ├─ build        Full pipeline: import → validate → optimize │
│  └─ watch        File watcher for live re-import            │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│                     Core Components                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ImportEngine       Reads GLB/FBX via AssimpNet             │
│  ├─ MeshImporter    Parse geometry, bones, animations       │
│  ├─ TextureImporter Extract embedded textures              │
│  └─ MetadataParser  Read Sketchfab metadata                │
│                                                               │
│  ValidationEngine   Validate imported data                   │
│  ├─ MeshValidator   Polycount, scale, normals              │
│  ├─ TextureValidator Dimensions, format, channels           │
│  └─ ReferenceValidator Check material/prefab links         │
│                                                               │
│  OptimizationEngine Generate LOD variants                    │
│  ├─ MeshSimplifier  Decimation algorithm (FastQuadricMesh) │
│  ├─ TextureCompressor PNG/DDS, mipmap generation           │
│  └─ LODGenerator     Create LOD0/LOD1/LOD2 meshes           │
│                                                               │
│  GeneratorEngine    Create Unity artifacts                   │
│  ├─ PrefabGenerator Serialize prefabs (.prefab binary)      │
│  ├─ MaterialGenerator Create faction color materials         │
│  ├─ AddressablesGenerator Build catalog entries             │
│  └─ DefinitionUpdater Inject visual_asset into YAML         │
│                                                               │
│  BuildEngine        Orchestrate full pipeline                │
│  ├─ PipelineExecutor Chain processors, handle errors         │
│  ├─ Logger          Structured logging (Serilog)            │
│  └─ Reporter        Build reports (JSON, HTML)              │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│                  Intermediate Formats                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ImportedAsset (in-memory)                                   │
│  ├─ MeshData          Vertices, indices, bones, animations  │
│  ├─ TextureData       Pixel data, metadata                  │
│  ├─ MaterialData      Shader refs, properties               │
│  └─ Metadata          Polycount, bounds, import settings    │
│                                                               │
│  OptimizedAsset (in-memory)                                  │
│  ├─ LODVariants       LOD0, LOD1, LOD2 mesh data            │
│  ├─ CompressedTextures DDS/PNG alternatives                │
│  └─ Metadata          Final polycount per LOD               │
│                                                               │
│  UnityAsset (serialized .prefab)                            │
│  ├─ GameObject tree   Root + child transforms               │
│  ├─ MeshFilter        References optimized LOD meshes       │
│  ├─ Materials         Faction colors, emission              │
│  └─ LODGroup          Screen percentage thresholds           │
│                                                               │
├─────────────────────────────────────────────────────────────┤
│                   Configuration (YAML)                       │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  asset_pipeline.yaml (per-pack)                            │
│  └─ Assets:                                                 │
│      ├─ Models:                                             │
│      │   sw_clone_trooper_phase2:                           │
│      │     source: assets/raw/sw_clone_trooper_phase2_sketchfab_001/model.glb
│      │     faction: republic                                │
│      │     type: infantry                                   │
│      │     scale: 1.0                                       │
│      │     lod:                                             │
│      │       enabled: true                                  │
│      │       levels: [100, 60, 30]  # percent polycount     │
│      │     addressable: sw-clone-trooper-republic           │
│      │     output: prefabs/Clone_Trooper_Republic.prefab    │
│      └─ ... (9 models for v0.7.0)                          │
│                                                               │
│      └─ Materials:                                          │
│          republic_blue:                                     │
│            color: "#4488FF"                                 │
│            emission: "#2244FF"                              │
│            intensity: 1.5                                   │
│          cis_orange:                                        │
│            color: "#FF4400"                                 │
│            emission: "#FF2200"                              │
│            intensity: 1.5                                   │
│                                                               │
│  schema: asset_pipeline.schema.json (JSON Schema)          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1, ~40 hours)

**Create `src/Tools/AssetPipeline/`** project (net8.0 console app)

```csharp
src/Tools/AssetPipeline/
├── Program.cs                    # CLI entry, command routing
├── Commands/
│   ├── ImportCommand.cs          # dotnet run -- import <pack-path>
│   ├── ValidateCommand.cs        # dotnet run -- validate <pack-path>
│   ├── OptimizeCommand.cs        # dotnet run -- optimize <pack-path>
│   ├── GenerateCommand.cs        # dotnet run -- generate <pack-path>
│   └── BuildCommand.cs           # dotnet run -- build <pack-path> (full pipeline)
├── Engines/
│   ├── ImportEngine.cs           # Import GLB/FBX via AssimpNet
│   ├── ValidationEngine.cs       # Validate imported data
│   ├── OptimizationEngine.cs     # Generate LOD variants
│   └── GenerationEngine.cs       # Create Unity artifacts
├── Processors/
│   ├── MeshProcessor.cs          # Mesh decimation, optimization
│   ├── TextureProcessor.cs       # Compression, mipmap generation
│   ├── MaterialProcessor.cs      # Create faction color materials
│   ├── PrefabProcessor.cs        # Serialize .prefab files
│   └── AddressablesProcessor.cs  # Generate catalog entries
├── Models/
│   ├── ImportedAsset.cs          # In-memory imported data
│   ├── OptimizedAsset.cs         # Optimized (LOD) data
│   ├── UnityAsset.cs             # Serialized artifact
│   ├── AssetConfig.cs            # YAML config model
│   └── ProcessingReport.cs       # Build report
├── Services/
│   ├── AssetPipelineService.cs   # Orchestrator
│   ├── LoggingService.cs         # Serilog wrapper
│   └── ReportingService.cs       # JSON/HTML reports
├── Validators/
│   ├─ MeshValidator.cs           # Polycount, scale rules
│   ├─ TextureValidator.cs        # Dimension, format rules
│   └─ ConfigValidator.cs         # YAML schema validation
├── asset_pipeline.schema.json    # JSON Schema
└── AssetPipeline.csproj

Tests/
└── AssetPipelineTests.cs         # Unit tests for pipeline
```

**Key Dependencies** (via NuGet):
- **AssimpNet** (MIT) - GLB/FBX parsing
- **FastQuadricMeshSimplifier** (MIT) - LOD mesh decimation
- **SixLabors.ImageSharp** (Six Labors Split License) - Texture processing
- **SixLabors.ImageSharp.Drawing** - Advanced image operations
- **Serilog** + **Serilog.Sinks.File** - Logging
- **System.CommandLine** - CLI argument parsing
- **YamlDotNet** - Config parsing
- **NJsonSchema** - JSON Schema validation
- **Spectre.Console** - Rich terminal output

---

### Phase 2: Integration with PackCompiler (Week 2, ~20 hours)

**Extend PackCompiler** to include:

```bash
dotnet run --project src/Tools/PackCompiler -- assets import packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets validate packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets build packs/warfare-starwars
```

Add to PackCompiler:
```csharp
var assetsImportCommand = new Command("import") { Description = "Import 3D models for a pack" };
assetsImportCommand.SetAction(parseResult => { /* delegate to AssetPipeline */ });

var assetsBuildCommand = new Command("build") { Description = "Build all assets for a pack" };
assetsBuildCommand.SetAction(parseResult => { /* full pipeline */ });
```

Result: Unified tool for packs AND assets

---

### Phase 3: v0.7.0 Asset Ingestion (Week 2-3, ~25 hours)

**Run pipeline on v0.7.0 models**:

```bash
# Configuration file
cat packs/warfare-starwars/asset_pipeline.yaml

# Full pipeline: import → validate → optimize → generate
dotnet run --project src/Tools/AssetPipeline -- build packs/warfare-starwars --phase v0.7.0

# Produces:
# ├─ packs/warfare-starwars/assets/imported/     (intermediate JSON)
# ├─ packs/warfare-starwars/assets/optimized/    (LOD meshes)
# ├─ Assets/warfare-starwars/models/             (prefabs, materials)
# ├─ Assets/warfare-starwars/addressables.yaml   (catalog)
# └─ build_report_v0_7_0.json                    (detailed report)
```

---

### Phase 4: Testing & Documentation (Week 3, ~15 hours)

**Test Suite**:
```csharp
[Fact]
public void Import_CloneTrooper_SucceedsWithCorrectPolycount()
{
    // Given: raw GLB file
    // When: import engine processes it
    // Then: polycount matches expected (35.6k)
}

[Fact]
public void LODGeneration_CreatesThreeLevels()
{
    // LOD0: 100%, LOD1: 60%, LOD2: 30%
}

[Fact]
public void Prefab_CreationMatchesVFXFactory_Pattern()
{
    // Serialized .prefab matches expected binary format
}

[Fact]
public void Addressables_CatalogEntriesValid()
{
    // sw-clone-trooper-republic, etc.
}
```

**Documentation**:
- ASSET_PIPELINE_CLI.md (user guide)
- ASSET_PIPELINE_SCHEMA.md (configuration reference)
- EXTENDING_ASSET_PIPELINE.md (custom processor guide)

---

## Key Design Decisions

### 1. Why AssimpNet?

| Consideration | Choice | Why |
|---------------|--------|-----|
| GLB/FBX parsing | AssimpNet | Industry standard, stable, cross-platform |
| Alternative | FBX SDK | Too heavy, Windows-only, licensing complexity |
| Alternative | Custom parser | Too much work, reinventing the wheel |

### 2. Why LOD via FastQuadricMeshSimplifier?

| Consideration | Choice | Why |
|---------------|--------|-----|
| LOD generation | FastQuadricMeshSimplifier | Best quality/speed ratio, proven in games |
| Alternative | Blender script | External dependency, harder to automate |
| Alternative | Unity Editor API | Not available outside editor |

### 3. Why Declarative Config (YAML)?

```yaml
assets:
  models:
    sw_clone_trooper_phase2:
      source: assets/raw/.../model.glb
      lod:
        levels: [100, 60, 30]
```

Instead of hardcoding in C#:
- **Reusable** — same engine handles all packs
- **Discoverable** — `asset_pipeline.yaml` documents what's imported
- **Versioned** — config changes tracked in git
- **Validated** — schema ensures correctness before processing

### 4. Intermediate Formats

```
Raw GLB → ImportedAsset (JSON) → OptimizedAsset (JSON) → UnityAsset (.prefab)
```

**Why**:
- Cacheable (skip re-import if raw unchanged)
- Debuggable (inspect JSON at each step)
- Testable (mock ImportedAsset for testing optimization)

---

## Extensibility Points

### Custom Processors

Users can extend pipeline with custom processors:

```csharp
public interface IAssetProcessor
{
    Task<ProcessingResult> ProcessAsync(ImportedAsset asset, AssetConfig config);
}

public class CustomLightsaberGlowProcessor : IAssetProcessor
{
    public async Task<ProcessingResult> ProcessAsync(ImportedAsset asset, AssetConfig config)
    {
        // Custom logic: add glow, emit trails, etc.
        return result;
    }
}

// Register in DI container
services.AddAssetProcessor<CustomLightsaberGlowProcessor>();
```

### Custom Validators

```csharp
public interface IAssetValidator
{
    ValidationResult Validate(ImportedAsset asset, AssetConfig config);
}

public class StarWarsColorValidator : IAssetValidator
{
    public ValidationResult Validate(ImportedAsset asset, AssetConfig config)
    {
        // Ensure faction colors match palette
    }
}
```

### Custom Output Formats

```csharp
public interface IAssetExporter
{
    Task ExportAsync(OptimizedAsset asset, string outputPath);
}

public class FBXExporter : IAssetExporter { }
public class GltfExporter : IAssetExporter { }
public class UnrealUAssetExporter : IAssetExporter { }
```

---

## Testing Strategy

### Unit Tests
```
AssetPipelineTests.cs
├─ ImportEngine_Tests.cs
│  ├─ Import_GLB_Succeeds
│  ├─ Import_InvalidFormat_Fails
│  └─ Import_Polycount_MatchesExpected
├─ OptimizationEngine_Tests.cs
│  ├─ LODGeneration_ProducesThreeLevels
│  ├─ LODGeneration_Maintains_PolyPercentages
│  └─ TextureCompression_Succeeds
├─ GenerationEngine_Tests.cs
│  ├─ PrefabGeneration_CreatesValidBinary
│  ├─ MaterialGeneration_AppliesFactionColors
│  └─ AddressablesGeneration_ValidEntries
└─ Integration_Tests.cs
   ├─ FullPipeline_v0_7_0_Models_Succeeds
   ├─ FullPipeline_Output_Matches_Expected
   └─ FullPipeline_Performance_Under_5min
```

### Performance Targets
- Import single model: &lt; 5 sec
- LOD generation (5k → 1.5k): &lt; 10 sec
- Prefab generation: &lt; 2 sec
- **Full pipeline (9 models)**: &lt; 5 minutes

---

## v0.7.0 Configuration Example

**`packs/warfare-starwars/asset_pipeline.yaml`**:

```yaml
version: 1.0
pack_id: warfare-starwars
target_unity_version: 2021.3.45f2

materials:
  republic:
    faction: republic
    color: "#4488FF"
    emission: "#2244FF"
    intensity: 1.5
  cis:
    faction: cis
    color: "#FF4400"
    emission: "#FF2200"
    intensity: 1.5

assets:
  v0_7_0_critical:
    - id: clone_trooper_phase2
      source: assets/raw/sw_clone_trooper_phase2_sketchfab_001/model.glb
      type: infantry
      faction: republic
      scale: 1.0
      lod:
        enabled: true
        levels: [100, 60, 30]
      material: republic
      addressable: sw-clone-trooper-republic
      output: prefabs/Clone_Trooper_Republic.prefab
      update_definition: true
      definition_path: units/republic_units.yaml
      definition_id: clone_trooper

    - id: general_grievous
      source: assets/raw/sw_general_grievous_sketchfab_001/model.glb
      type: hero
      faction: cis
      scale: 1.0
      lod:
        levels: [100, 60, 30]
      material: cis
      addressable: sw-general-grievous
      output: prefabs/General_Grievous_CIS.prefab
      update_definition: true
      definition_path: units/cis_units.yaml
      definition_id: general_grievous

    # ... (8 more models)

build:
  output_dir: Assets/warfare-starwars
  addressables_output: Assets/warfare-starwars/addressables.yaml
  generate_report: true
  report_format: [json, html]
```

---

## Success Metrics (v0.7.0)

- [x] AssetPipeline tool created and integrated with PackCompiler
- [ ] Full pipeline runs on 9 v0.7.0 models in &lt; 5 minutes
- [ ] All 9 prefabs generate with correct LOD variants
- [ ] Addressables catalog created with 10 entries
- [ ] Game definitions updated with visual_asset references
- [ ] 40+ tests covering all pipeline stages
- [ ] Comprehensive documentation for extending pipeline
- [ ] 35% asset coverage achieved (9 visible units)

---

## Timeline

- **Week 1**: Infrastructure (Commands, Engines, Models, Services)
- **Week 2**: Integration with PackCompiler + v0.7.0 ingestion
- **Week 3**: Testing, documentation, v0.8.0 preparation
- **Week 4-7**: v0.8.0 full automation (same tool, 4 models)

---

## Extensibility for v0.9.0 + v1.0.0

Once v0.7.0 is done, pipeline scales to:

**v0.9.0** (19 building models):
```bash
dotnet run --project src/Tools/PackCompiler -- assets build packs/warfare-starwars --phase v0.9.0
```

**New packs** (warfare-modern, warfare-guerrilla, economy-pack):
```bash
dotnet run --project src/Tools/PackCompiler -- assets build packs/warfare-modern
```

**Custom processors**:
- Building construction animations
- Faction-specific paint jobs
- Damage variants
- Animation blending

---

## Conclusion

This design creates a **reusable, testable, extensible asset pipeline** that:

✅ Eliminates manual Unity Editor work
✅ Supports all future packs (v0.8.0 → v1.0.0)
✅ Follows DINOForge patterns (CLI, declarative config, testing)
✅ Is fully agent-automatable (no GUI)
✅ Scales to 100+ models across 5+ packs
✅ Can be extended with custom processors

**Ready to implement?**
