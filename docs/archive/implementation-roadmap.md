# v0.7.0 + v0.8.0 Implementation Roadmap

**Status**: ✅ Design & Planning Complete. Ready for Development Phase.
**Date**: 2026-03-13
**Target**: v0.7.0 release in 3-4 weeks, v0.8.0 in 6-8 weeks

---

## What's Been Completed (Before Development)

### Phase 0: Preparation (✅ COMPLETE)

**Assets & Infrastructure** (12 hours)
- ✅ 9 real Star Wars models downloaded (70MB)
- ✅ All CC-BY-4.0 licensed and verified
- ✅ Stored in `packs/warfare-starwars/assets/raw/` with manifests

**Design & Governance** (8 hours)
- ✅ Unified asset system architecture designed
- ✅ CLAUDE.md updated with asset pipeline governance
- ✅ Schema created (`asset_pipeline.schema.json`)
- ✅ Configuration created (`asset_pipeline.yaml`)
- ✅ Documentation completed (UNIFIED_ASSET_SYSTEM.md)

**Current State**:
- ✅ 678 tests passing
- ✅ Build clean (0 errors)
- ✅ Git history clean (proper commits)
- ✅ No blockers identified

---

## Development Phase (Weeks 1-7, ~90 hours)

### Week 1: Asset Pipeline Infrastructure (40 hours)

**Objective**: Build unified asset import pipeline in PackCompiler

**Tasks**:

1. **Create AssetCommands.cs** (8 hours)
   - `ImportCommand` — `assets import <pack>`
   - `ValidateCommand` — `assets validate <pack>`
   - `OptimizeCommand` — `assets optimize <pack>`
   - `GenerateCommand` — `assets generate <pack>`
   - `BuildCommand` — `assets build <pack>` (orchestrator)
   - Wire into Program.cs command routing

2. **Create AssetImportService** (8 hours)
   - Parse GLB/FBX via **AssimpNet**
   - Extract mesh data (vertices, indices, bones)
   - Extract texture data
   - Serialize to ImportedAsset (JSON)
   - Unit tests: 4+ tests

3. **Create AssetOptimizationService** (8 hours)
   - Mesh decimation via **FastQuadricMeshSimplifier**
   - Generate LOD0, LOD1, LOD2 variants
   - Preserve UV seams, bone weights
   - Output OptimizedAsset (JSON)
   - Unit tests: 4+ tests

4. **Create PrefabGenerationService** (8 hours)
   - Serialize OptimizedAsset → .prefab binary format
   - Create LODGroup component
   - Add MeshFilter with LOD references
   - Apply faction color materials
   - Unit tests: 4+ tests

5. **Create AddressablesService** (8 hours)
   - Generate Addressables catalog entries
   - Build YAML output for runtime loading
   - Link to prefab paths
   - Validate address uniqueness
   - Unit tests: 2+ tests

**Deliverables**:
- ✅ Unified `dotnet run -- assets` command group
- ✅ All services implemented and unit tested
- ✅ Can import single model end-to-end
- ✅ Generates valid .prefab files
- ✅ Creates Addressables catalog

**Git Commit**: `feat: implement asset pipeline (import, optimize, generate) in PackCompiler`

---

### Week 2: v0.7.0 Asset Ingestion (30 hours)

**Objective**: Run pipeline on all 9 v0.7.0 models, verify outputs

**Tasks**:

1. **Test Import Stage** (5 hours)
   - `assets import packs/warfare-starwars --phase v0.7.0`
   - Verify 9 models parse correctly
   - Check polycount matches expected
   - Verify bone/weight data preserved
   - Create intermediate JSON snapshots for regression testing

2. **Test Optimize Stage** (5 hours)
   - `assets optimize packs/warfare-starwars --phase v0.7.0`
   - Verify LOD0/LOD1/LOD2 generate for all 9
   - Confirm polycount percentages match config (100%, 60%, 30%)
   - Check no vertex seam breaks, UV islands preserved
   - Performance: &lt; 10 sec per model

3. **Test Generate Stage** (5 hours)
   - `assets generate packs/warfare-starwars --phase v0.7.0`
   - Verify 9 .prefab files created
   - Check LODGroup components present
   - Verify materials applied correctly
   - Check output paths match config

4. **Test Build (Full Pipeline)** (5 hours)
   - `assets build packs/warfare-starwars --phase v0.7.0`
   - End-to-end test: raw GLB → .prefab in &lt; 5 min
   - Verify Addressables catalog created
   - Check all outputs present and valid
   - Generate build report (JSON + HTML)

5. **Definition Updates** (5 hours)
   - Implement DefinitionUpdateService
   - Auto-inject `visual_asset: sw-*` into YAML definitions
   - Verify game definitions updated correctly
   - Unit tests: 2+ tests

6. **Integration Tests** (5 hours)
   - `AssetPipelineTests.cs` with comprehensive test suite
   - Test v0.7.0 models end-to-end
   - Test output validity (prefabs load in inspector)
   - Test regression (v0.6.0 models still work)
   - Test v0.8.0 config parsing (ready for next phase)

**Deliverables**:
- ✅ Full pipeline runs on 9 v0.7.0 models
- ✅ All prefabs generated and valid
- ✅ Addressables catalog correct
- ✅ Game definitions updated
- ✅ 20+ integration tests
- ✅ Build report generated (JSON + HTML)

**Git Commit**: `feat: complete asset pipeline for v0.7.0 (9 models, 35% coverage)`

---

### Week 3: Testing, Verification & v0.8.0 Prep (20 hours)

**Objective**: Verify in-game functionality, prepare v0.8.0

**Tasks**:

1. **In-Game Verification** (8 hours)
   - Copy generated prefabs to Unity project
   - Load in scene
   - Verify visual appearance
   - Test LOD transitions at different distances
   - Check 60 FPS @ 16 units benchmark
   - Verify faction colors apply correctly
   - Check AT-TE mapping fix works
   - Document any visual issues

2. **Performance Optimization** (4 hours)
   - Profile import/optimize/generate stages
   - Optimize hot paths (mesh decimation)
   - Improve LOD generation algorithm if needed
   - Ensure full pipeline &lt; 5 min for 9 models
   - Target: &lt; 30 sec per model

3. **Documentation** (4 hours)
   - Create ASSET_PIPELINE_CLI.md (command reference)
   - Update CLAUDE.md governance if needed
   - Add XML docs to all public services
   - Create troubleshooting guide

4. **v0.8.0 Preparation** (4 hours)
   - Verify v0.8.0 phase in asset_pipeline.yaml is valid
   - Run schema validation on full config
   - Test pipeline with v0.8.0 phase (dry run)
   - Document any phase-specific notes

**Deliverables**:
- ✅ v0.7.0 verified in-game
- ✅ Performance targets met
- ✅ Full documentation complete
- ✅ v0.8.0 ready to execute

**Git Commit**: `docs: add asset pipeline documentation and complete v0.7.0 verification`

---

### Weeks 4-7: v0.8.0 Implementation (Optional Parallel)

**Objective**: Extend with 4 elite models, achieve 48% coverage

**Note**: Can run in parallel with v0.7.0 or sequentially. Same pipeline, different config phase.

**Tasks**:

1. **Run v0.8.0 Pipeline** (10 hours)
   - `assets build packs/warfare-starwars --phase v0.8.0`
   - Same process as v0.7.0
   - Import, optimize, generate 4 elite models
   - Special attention to Droideka (257k poly - high detail)
   - Attention to AAT Tank (4k poly - ultra low)

2. **Elite Unit Testing** (10 hours)
   - Verify all 4 prefabs generate
   - Test Droideka LOD aggressiveness
   - Verify AAT Tank maintains quality at low poly
   - Test 60 FPS @ 32 units benchmark
   - Full integration tests

3. **Documentation & Polish** (10 hours)
   - Update coverage matrix (48%)
   - Release notes
   - Known issues
   - Performance characteristics

4. **Cleanup** (10 hours)
   - Refactor shared code (LOD strategy)
   - Create reusable processor templates
   - Establish patterns for v0.9.0

**Deliverables**:
- ✅ 4 elite models integrated
- ✅ 48% coverage achieved
- ✅ Full test coverage
- ✅ Ready for v0.9.0 building models

---

## What v0.7.0 Solves

### Assets (35% Coverage)
- ✅ Clone Trooper Phase II (replaces helmet placeholder)
- ✅ General Grievous (CIS hero)
- ✅ B2 Super Droid (heavy unit)
- ✅ AT-TE Walker (**fixes V-19 mapping bug**)
- ✅ Jedi Temple (**first building visual**)

### Gameplay
- ✅ Both heroes visible (Clone + Grievous)
- ✅ Full infantry unit visible
- ✅ Heavy unit visible
- ✅ Vehicle visible (correct model)
- ✅ First building visual

### Infrastructure
- ✅ Unified asset pipeline tool
- ✅ Declarative configuration system
- ✅ Automated LOD generation
- ✅ Prefab serialization
- ✅ Addressables integration
- ✅ Game definition auto-updates

---

## Dependencies (NuGet)

Add to `src/Tools/PackCompiler/PackCompiler.csproj`:

```xml
<!-- 3D Model Import -->
<PackageReference Include="AssimpNet" Version="5.0.0" />

<!-- Mesh Optimization & LOD -->
<PackageReference Include="FastQuadricMeshSimplifier" Version="1.0.0" />

<!-- Image Processing (future texture handling) -->
<PackageReference Include="SixLabors.ImageSharp" Version="3.0.2" />
<PackageReference Include="SixLabors.ImageSharp.Drawing" Version="2.1.0" />

<!-- Already have: Serilog, YamlDotNet, NJsonSchema, System.CommandLine, Spectre.Console -->
```

---

## Testing Strategy

### Unit Tests (40+ tests)
- AssetImportService: 4 tests (parse, verify polycount, bones, UV)
- AssetOptimizationService: 4 tests (LOD generation, percentages, quality)
- PrefabGenerationService: 4 tests (serialization, LODGroup, materials)
- AddressablesService: 2 tests (catalog, uniqueness, paths)
- DefinitionUpdateService: 2 tests (YAML injection, validation)
- AssetValidator: 4 tests (polycount rules, scale, references)
- AssetConfigValidator: 4 tests (schema, required fields, enums)

### Integration Tests (20+ tests)
- FullPipeline_V0_7_0_Models_Succeed (import → build)
- FullPipeline_Output_Matches_Expected (prefab integrity)
- FullPipeline_Performance_Under_5min (9 models)
- Regression_V0_6_0_Models_Still_Work (backward compat)
- V0_8_0_Config_ParsesCorrectly (forward compat)

### In-Game Tests (Manual)
- Prefabs load in scene
- LOD transitions smooth (no pop-in)
- Faction colors apply
- 60 FPS @ 16 units (v0.7.0)
- 60 FPS @ 32 units (v0.8.0)
- AT-TE mapping correct

---

## Success Criteria

**v0.7.0** (End of Week 3):
- [x] 9 models configured in asset_pipeline.yaml
- [ ] Asset pipeline infrastructure complete
- [ ] Full pipeline runs in &lt; 5 minutes
- [ ] 5 prefabs generated without errors
- [ ] Addressables catalog created (10 entries)
- [ ] Game definitions auto-updated
- [ ] 35% coverage achieved
- [ ] Both heroes visible
- [ ] First building visual working
- [ ] AT-TE mapping fixed
- [ ] 60 FPS @ 16 units
- [ ] 40+ unit tests passing
- [ ] 20+ integration tests passing

**v0.8.0** (End of Week 7, optional):
- [ ] 4 elite models integrated
- [ ] 48% coverage achieved
- [ ] 60 FPS @ 32 units
- [ ] All tests passing
- [ ] Documentation complete

---

## Timeline

```
Week 1: Asset Pipeline Infrastructure (40 hrs)
  Day 1-2: Commands + routing
  Day 3-4: Import service
  Day 5: Optimize service
  Day 6-7: Generate + Addressables services

Week 2: v0.7.0 Ingestion (30 hrs)
  Day 1: Test import stage (5 hrs)
  Day 2: Test optimize stage (5 hrs)
  Day 3: Test generate stage (5 hrs)
  Day 4: Test build (5 hrs)
  Day 5: Definition updates (5 hrs)
  Day 6-7: Integration tests (5 hrs)

Week 3: Verification & v0.8.0 Prep (20 hrs)
  Day 1-2: In-game verification (8 hrs)
  Day 3: Performance optimization (4 hrs)
  Day 4: Documentation (4 hrs)
  Day 5-7: v0.8.0 prep (4 hrs)

Week 4-7: v0.8.0 (Optional, 40 hrs if parallel)
  Same structure, 4 models instead of 9

Total: 90 hours = 6-8 weeks elapsed, 3-4 weeks focused work
```

---

## Repository State After Completion

```
src/Tools/PackCompiler/
├── Commands/
│   ├── AssetCommands.cs (NEW)
│   ├── PackCommands.cs (existing)
│   └── ... (other commands)
├── Services/ (NEW)
│   ├── AssetImportService.cs
│   ├── AssetOptimizationService.cs
│   ├── PrefabGenerationService.cs
│   ├── AddressablesService.cs
│   └── DefinitionUpdateService.cs
├── Models/ (NEW)
│   ├── ImportedAsset.cs
│   ├── OptimizedAsset.cs
│   ├── AssetConfig.cs
│   └── ProcessingReport.cs
├── Validators/ (NEW)
│   ├── AssetValidator.cs
│   ├── ConfigValidator.cs
│   └── OutputValidator.cs
└── Program.cs (updated)

schemas/
├── asset_pipeline.schema.json (NEW)
└── ... (existing schemas)

packs/warfare-starwars/
├── asset_pipeline.yaml (NEW)
├── assets/
│   └── raw/ (existing, 13 models)
└── units/, buildings/ (definitions updated)

Assets/warfare-starwars/ (Generated by pipeline)
├── models/
├── prefabs/ (9 .prefab files)
├── materials/ (faction colors)
└── addressables.yaml

Tests/
├── AssetPipelineTests.cs (NEW, 40+ tests)
├── Integration/ (20+ integration tests)
└── ... (existing tests)

Documentation/
├── ASSET_PIPELINE_CLI.md (NEW)
├── UNIFIED_ASSET_SYSTEM.md (✅ complete)
├── IMPLEMENTATION_ROADMAP.md (this file, NEW)
├── CLAUDE.md (updated with governance)
└── ... (existing docs)
```

---

## Next Action

**Ready to start Week 1 development?**

**Command to begin**:
```bash
dotnet build src/DINOForge.sln  # Verify clean build
dotnet test src/DINOForge.sln   # Verify tests pass (should be 678)
```

Then start implementing `src/Tools/PackCompiler/Commands/AssetCommands.cs`

---

**Prepared by**: Claude Haiku 4.5
**Status**: Ready for development
**Confidence**: HIGH (architecture sound, all dependencies verified, tests planned)
**Estimated Delivery**: v0.7.0 in 3-4 weeks, v0.8.0 in 6-8 weeks
