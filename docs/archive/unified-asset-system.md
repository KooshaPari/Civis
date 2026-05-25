# Unified Asset System - Architecture & Governance

**Strategy**: Extend **PackCompiler** (single source of truth for pack operations) to handle all asset workflows
**Governance**: CLAUDE.md + Skills define workflows, validation, and agent responsibilities
**Status**: Design complete, ready to implement in Week 1

---

## Current Fragmentation Problem

```
Multiple tools, fragmented workflows:

VFXPrefabGenerator (Unity Editor, net472)
  ├─ Generates 11 VFX prefabs
  ├─ Hardcoded catalog
  └─ Manual Unity Editor invocation

PackCompiler (net8.0 CLI)
  ├─ validate packs/
  ├─ build packs/
  ├─ assets inspect <bundle>
  ├─ assets validate <bundle>
  └─ ... (40+ scattered functions)

Cli Tool
  ├─ Game status queries
  ├─ Package overrides
  ├─ Reload triggers
  └─ ... (separate project)

DumpTools
  ├─ Entity analysis
  └─ Component inspection

Custom scripts (download_models_web.py, etc.)
```

**Problem**: No unified entry point for asset workflows → agents get confused → fragmented knowledge

**Solution**:
```
PackCompiler (unified entry point)
  ├─ pack commands (validate, build, add, remove)
  ├─ assets commands (import, validate, optimize, generate, build)
  ├─ content commands (validate, schema, generate)
  ├─ vfx commands (generate, inspect, validate)
  └─ sync commands (download, update, verify)
```

---

## New CommandLine Interface

### Current (Scattered)

```bash
dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars

dotnet run --project src/Tools/VFXPrefabGenerator -- [no CLI, manual invocation]

python scripts/download_models_web.py [separate script]

dotnet run --project src/Tools/Cli -- query game status [separate tool]
```

### New (Unified)

```bash
# Pack operations (existing, works)
dotnet run --project src/Tools/PackCompiler -- pack validate packs/warfare-starwars

# Asset import pipeline (NEW)
dotnet run --project src/Tools/PackCompiler -- assets import packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets validate packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets optimize packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets generate packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- assets build packs/warfare-starwars

# Content sync (models, textures, configs)
dotnet run --project src/Tools/PackCompiler -- sync download packs/warfare-starwars --phase v0.7.0
dotnet run --project src/Tools/PackCompiler -- sync verify packs/warfare-starwars

# VFX generation (integrate existing tool)
dotnet run --project src/Tools/PackCompiler -- vfx generate packs/warfare-starwars
dotnet run --project src/Tools/PackCompiler -- vfx validate packs/warfare-starwars

# Content schema generation
dotnet run --project src/Tools/PackCompiler -- schema generate assets/unit.schema.json
```

---

## Architecture: Asset Import Pipeline in PackCompiler

```
src/Tools/PackCompiler/
├── Program.cs (main command router)
├── Commands/
│   ├── PackCommands.cs (existing)
│   ├── AssetCommands.cs (NEW)
│   │   ├─ ImportCommand
│   │   ├─ ValidateCommand
│   │   ├─ OptimizeCommand
│   │   ├─ GenerateCommand
│   │   └─ BuildCommand
│   ├── SyncCommands.cs (NEW)
│   │   ├─ DownloadCommand
│   │   └─ VerifyCommand
│   └── VfxCommands.cs (NEW, wraps VFXPrefabGenerator)
│       ├─ GenerateCommand
│       └─ ValidateCommand
│
├── Services/ (NEW)
│   ├── AssetImportService.cs
│   ├── AssetValidationService.cs
│   ├── AssetOptimizationService.cs
│   ├── PrefabGenerationService.cs
│   └── AddressablesService.cs
│
├── Models/ (NEW)
│   ├── ImportedAsset.cs
│   ├── OptimizedAsset.cs
│   ├── AssetConfig.cs
│   └── ProcessingReport.cs
│
└── Validators/ (NEW)
    ├── AssetValidator.cs
    ├── ConfigValidator.cs
    └── OutputValidator.cs
```

---

## Configuration: asset_pipeline.yaml

Every pack has `asset_pipeline.yaml` defining all asset operations:

**`packs/warfare-starwars/asset_pipeline.yaml`** (v0.7.0 example):

```yaml
version: 1.0
pack_id: warfare-starwars
target_unity_version: 2021.3.45f2

# Global asset settings
asset_settings:
  base_path: assets
  output_path: ../../Assets/warfare-starwars
  materials_path: materials
  texture_quality: high
  lod_strategy: aggressive  # aggressive | balanced | conservative

# Material definitions (faction colors)
materials:
  republic:
    faction: republic
    base_color: "#4488FF"
    emission_color: "#2244FF"
    emission_intensity: 1.5
    roughness: 0.4
    metallic: 0.2
  cis:
    faction: cis
    base_color: "#FF4400"
    emission_color: "#FF2200"
    emission_intensity: 1.5
    roughness: 0.5
    metallic: 0.3

# Asset import phases
phases:
  v0_7_0_critical:
    description: "Core units, heroes, vehicles, and first building"
    models:
      - id: clone_trooper_phase2
        file: raw/sw_clone_trooper_phase2_sketchfab_001/model.glb
        faction: republic
        type: infantry
        polycount_target: 35600
        scale: 1.0
        lod:
          enabled: true
          levels: [100, 60, 30]
          screen_sizes: [100, 50, 20]
        material: republic
        addressable_key: sw-clone-trooper-republic
        output_prefab: prefabs/Clone_Trooper_Republic.prefab
        update_definition:
          enabled: true
          file: units/republic_units.yaml
          id: clone_trooper
          field: visual_asset

      - id: general_grievous
        file: raw/sw_general_grievous_sketchfab_001/model.glb
        faction: cis
        type: hero
        polycount_target: 4500
        scale: 1.0
        lod:
          enabled: true
          levels: [100, 60, 30]
        material: cis
        addressable_key: sw-general-grievous
        output_prefab: prefabs/General_Grievous_CIS.prefab
        update_definition:
          enabled: true
          file: units/cis_units.yaml
          id: general_grievous

      # ... (8 more models for v0.7.0)

  v0_8_0_elite:
    description: "Elite units, specialized, vehicles"
    models:
      # ... (4 elite models)

  v0_9_0_future:
    description: "19 building models"
    models: []  # Define when buildings acquired

# Build outputs
build:
  output_directory: ../../Assets/warfare-starwars
  addressables_output: ../../Assets/warfare-starwars/addressables.yaml
  log_file: build_report.json
  generate_html_report: true
  performance_targets:
    import_time_sec: 5
    lod_generation_sec: 10
    prefab_generation_sec: 2
    total_pipeline_sec: 60
```

---

## Governance in CLAUDE.md

Add to CLAUDE.md (new section):

```markdown
## Asset Pipeline Governance

### Asset Workflows (Unified in PackCompiler)

Agents performing asset work MUST follow this sequence:

1. **Define** — Create/update `asset_pipeline.yaml` in pack root
2. **Download** — Run `dotnet run --project src/Tools/PackCompiler -- sync download <pack>`
3. **Import** — Run `dotnet run --project src/Tools/PackCompiler -- assets import <pack>`
4. **Validate** — Run `dotnet run --project src/Tools/PackCompiler -- assets validate <pack>`
5. **Optimize** — Run `dotnet run --project src/Tools/PackCompiler -- assets optimize <pack>`
6. **Generate** — Run `dotnet run --project src/Tools/PackCompiler -- assets generate <pack>`
7. **Verify** — Run `dotnet run --project src/Tools/PackCompiler -- assets build <pack>`
8. **Commit** — Git commit build artifacts + updated definitions

### Asset Schema (asset_pipeline.schema.json)

All `asset_pipeline.yaml` files MUST validate against `schemas/asset_pipeline.schema.json`

Agents MUST NOT:
- Hardcode polycount targets in C#
- Manually edit game definitions after asset generation
- Skip asset validation steps
- Import models without updating definitions
- Create ad-hoc asset directories

### Extension Points

Custom asset processors can be registered via DI in PackCompiler:

```csharp
// In PackCompiler/Program.cs
services.AddAssetProcessor&lt;CustomLightsaberGlowProcessor&gt;();
services.AddAssetValidator&lt;StarWarsColorValidator&gt;();
services.AddAssetExporter&lt;CustomFormatExporter&gt;();
```

### Testing Requirements

All asset operations MUST be tested:
- Unit tests for import engine
- Validation tests for all asset types
- Integration tests for full pipeline
- Regression tests for known assets (v0.6.0 models)

### Documentation Requirements

When adding asset workflow features:
- Update this governance section
- Document command in `ASSET_PIPELINE_CLI.md`
- Add schema changes to `asset_pipeline.schema.json`
- Create test cases in `AssetPipelineTests.cs`
```

---

## Implementation Roadmap

### Week 1: Foundation (40 hours)

**Step 1: Create `AssetCommands.cs` in PackCompiler** (10 hours)
```csharp
public class AssetCommands
{
    public static Command CreateImportCommand() { }
    public static Command CreateValidateCommand() { }
    public static Command CreateOptimizeCommand() { }
    public static Command CreateGenerateCommand() { }
    public static Command CreateBuildCommand() { }
}
```

**Step 2: Create Service Layer** (15 hours)
```csharp
src/Tools/PackCompiler/Services/
├── AssetImportService.cs (AssimpNet wrapper)
├── AssetOptimizationService.cs (LOD generation via FastQuadricMesh)
├── PrefabGenerationService.cs (serialize .prefab)
└── AddressablesService.cs (catalog building)
```

**Step 3: Create Data Models & Validators** (10 hours)
```csharp
src/Tools/PackCompiler/Models/
├── AssetConfig.cs (asset_pipeline.yaml model)
├── ImportedAsset.cs (in-memory representation)
├── OptimizedAsset.cs (LOD variants)
└── ProcessingReport.cs (build output)

src/Tools/PackCompiler/Validators/
├── AssetConfigValidator.cs (schema validation)
├── AssetValidator.cs (polycount, scale checks)
└── OutputValidator.cs (prefab integrity)
```

**Step 4: Wire Commands in Program.cs** (5 hours)
```csharp
var assetsCommand = new Command("assets") { Description = "Asset import pipeline" };
assetsCommand.AddCommand(AssetCommands.CreateImportCommand());
assetsCommand.AddCommand(AssetCommands.CreateValidateCommand());
// etc.
rootCommand.AddCommand(assetsCommand);
```

### Week 2: v0.7.0 Execution (30 hours)

**Step 1: Create asset_pipeline.yaml for warfare-starwars** (3 hours)
**Step 2: Test each pipeline stage independently** (10 hours)
- Import → validate GLB parsing
- Optimize → validate LOD generation
- Generate → validate prefab serialization
- Build → end-to-end test

**Step 3: Run full pipeline on 9 v0.7.0 models** (5 hours)
**Step 4: Verify outputs** (5 hours)
- Check prefabs load in Unity
- Verify Addressables catalog correct
- Confirm game definitions updated
- Test 60 FPS @ 16 units

**Step 5: Integration tests** (7 hours)
```csharp
[Fact]
public async Task AssetPipeline_V0_7_0_Models_CompleteSuccessfully()
{
    // Given: asset_pipeline.yaml with 9 v0.7.0 models
    // When: assets build packs/warfare-starwars --phase v0.7.0
    // Then: 9 prefabs generated, addressables valid, definitions updated
}
```

### Week 3: Documentation & v0.8.0 Prep (20 hours)

**Step 1: Document asset pipeline CLI** (5 hours)
- `ASSET_PIPELINE_CLI.md` (command reference)
- `asset_pipeline.schema.json` (JSON Schema)

**Step 2: Update CLAUDE.md with governance** (5 hours)

**Step 3: Create skills for asset workflows** (5 hours)
```
Skills:
  - asset-import-start
  - asset-optimize-mesh
  - asset-generate-prefab
  - asset-build-full-pipeline
```

**Step 4: v0.8.0 configuration** (5 hours)
- Extend `asset_pipeline.yaml` with v0.8.0 phase
- Reuse same tool, same process

---

## Dependencies to Add

```xml
<!-- Add to src/Tools/PackCompiler/PackCompiler.csproj -->

<ItemGroup>
  <!-- 3D Model Import -->
  <PackageReference Include="AssimpNet" Version="5.0.0" />

  <!-- Mesh Optimization -->
  <PackageReference Include="FastQuadricMeshSimplifier" Version="1.0.0" />

  <!-- Image Processing -->
  <PackageReference Include="SixLabors.ImageSharp" Version="3.0.2" />
  <PackageReference Include="SixLabors.ImageSharp.Drawing" Version="2.1.0" />

  <!-- Already have -->
  <!-- Serilog, YamlDotNet, NJsonSchema, System.CommandLine, Spectre.Console -->
</ItemGroup>
```

---

## Success Metrics (v0.7.0)

- [x] All 9 models downloaded and verified
- [ ] `asset_pipeline.yaml` defines all 9 models
- [ ] `dotnet run -- assets import` completes in &lt; 5 sec per model
- [ ] `dotnet run -- assets optimize` generates valid LOD variants
- [ ] `dotnet run -- assets generate` creates 9 prefabs
- [ ] `dotnet run -- assets build` completes full pipeline in &lt; 5 min
- [ ] Addressables catalog has 10 entries
- [ ] Game definitions auto-updated with visual_asset refs
- [ ] 40+ unit tests for all stages
- [ ] v0.8.0 ready (4 models configured, same pipeline)
- [ ] CLAUDE.md updated with asset governance
- [ ] ASSET_PIPELINE_CLI.md complete reference

---

## Why This Approach (vs separate AssetPipeline tool)

| Decision | Benefit |
|----------|---------|
| **Extend PackCompiler (not new tool)** | Single source of truth, agents don't get confused, unified workflows |
| **Configuration as YAML** | Discoverable, versioned, validated, agent-friendly |
| **Governance in CLAUDE.md** | Clear responsibilities, testable expectations, prevents fragmentation |
| **Skills for workflows** | Agent-automatable, reproducible, composable |
| **Service layer** | Testable, reusable, extensible to other packs |
| **Declarative over imperative** | No hardcoding, scales to v0.9.0 + v1.0.0 |

---

## Timeline

- **Week 1**: Foundation (40 hrs) → asset import/optimize/generate infrastructure complete
- **Week 2**: v0.7.0 execution (30 hrs) → 9 models processed, tested, verified
- **Week 3**: Documentation + v0.8.0 (20 hrs) → CLAUDE.md updated, v0.8.0 ready
- **Total**: 90 hours = 6 weeks elapsed, ~3-4 weeks of focused work

---

## Next Steps

1. Confirm this approach ✓
2. Update CLAUDE.md with asset governance (Day 1)
3. Create asset_pipeline.yaml schema (Day 2)
4. Build AssetCommands.cs → Service layer (Week 1)
5. Test on 9 v0.7.0 models (Week 2)
6. Document + scale to v0.8.0 (Week 3)

**Ready to implement?**
