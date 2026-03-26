# Week 4: Real-World Testing & Optimization - PLAN

**Status**: Pipeline infrastructure complete, ready for production assets

---

## Current State

### What Works
- ✓ Asset pipeline fully implemented (import → validate → optimize → generate)
- ✓ 7 integration tests passing with synthetic data
- ✓ CLI commands wired and operational
- ✓ All services (import, validation, optimization, prefab generation, addressables, definitions)
- ✓ YAML configuration system working
- ✓ Build succeeds with 0 errors

### Blockers
- **Real model files missing**: Placeholder files in `packs/warfare-starwars/assets/raw/` are HTML, not GLB/FBX
  - Expected: Binary GLB/FBX files
  - Actual: HTML download error pages
  - Root cause: Model download script or Sketchfab fetch failed
  - Impact: Cannot run pipeline on real Star Wars models yet

---

## Week 4 Tasks

### 1. Acquire Real 3D Models
**Effort**: Medium (depends on external resources)
**Actions**:
- [ ] Download actual Star Wars Clone Wars models from Sketchfab or similar
- [ ] Verify GLB/FBX magic bytes (0x67 0x6C 0x54 0x46 for GLB, 0xFB 0x4B for FBX)
- [ ] Place in `packs/warfare-starwars/assets/raw/<model-id>/model.{glb,fbx}`
- [ ] Verify asset_pipeline.yaml file paths match downloaded models
- [ ] Test import on first model to validate pipeline doesn't hang

**Test Commands**:
```bash
# Verify file is valid GLB
hexdump -C packs/warfare-starwars/assets/raw/*/model.glb | head -1

# Test import on single model
dotnet run --project src/Tools/PackCompiler -- pipeline import packs/warfare-starwars
```

### 2. Debug Import Timeout
**Effort**: Low
**Issue**: Import command hangs even with empty config
**Root causes to investigate**:
1. AssimpNet initialization hanging on Windows
2. YAML deserialization infinite loop
3. File system operations blocking
4. Missing exception handling for HTML files

**Actions**:
- [ ] Add diagnostic logging to AssetImportService
- [ ] Add timeout handling in Program.cs
- [ ] Test with smallest possible model (~10KB GLB)
- [ ] Verify AssimpNet working with sample GLB

**Test**:
```bash
# Add timeout to pipeline import
dotnet run --project src/Tools/PackCompiler -- pipeline import packs/warfare-starwars --timeout 30
```

### 3. Performance Benchmarking
**Effort**: Medium
**Target**: 9 models &lt; 5 min total
**Metrics to track**:
- Import: &lt; 0.5s per model (parsing GLB/FBX)
- Validation: &lt; 0.1s per model (checking polycount/scale)
- Optimization: &lt; 1.0s per model (LOD generation via decimation)
- Generation: &lt; 0.5s per model (prefab + addressables)
- **Total**: &lt; 18s per model, ~2.7 min for 9 models

**Implementation**:
```csharp
// Add timing to each step
var swImport = Stopwatch.StartNew();
var imported = importService.ImportAsync(assetId, path).GetAwaiter().GetResult();
swImport.Stop();
AnsiConsole.WriteLine($"  Import: {swImport.ElapsedMilliseconds}ms");

// Report summary at end
AnsiConsole.MarkupLine($"Average time per asset: {totalMs / count}ms");
```

### 4. Visual Quality Assurance
**Effort**: Medium
**Process**:
1. Run pipeline on all 9 v0.7.0 models
2. Inspect generated prefab YAML (check LODGroup setup)
3. Import prefabs into Unity Editor
4. Verify LOD transitions visually
5. Check material assignments
6. Validate skeleton/rigging (if applicable)

**Outputs to verify**:
- `.prefab` files have valid YAML structure
- LOD0/1/2 mesh references are valid
- Material paths match output
- Addressables group GUIDs are unique
- Catalog entries complete

### 5. Definition Update Testing
**Effort**: Medium
**Test case**: Inject visual_asset refs into game definitions
**Steps**:
1. Create sample `units/republic_units.yaml` with test units
2. Configure asset_pipeline.yaml with `update_definition` enabled
3. Run pipeline build
4. Verify definition file updated with visual_asset field

**Expected output**:
```yaml
- id: clone_trooper
  name: Clone Trooper
  visual_asset: prefabs/Clone_Trooper_Republic.prefab  # Auto-injected
  health: 100
  armor: 10
```

### 6. Addressables Runtime Testing
**Effort**: High
**Scope**: Create minimal Unity project
**Process**:
1. Generate addressables catalog + groups
2. Copy to Unity project `Assets/` directory
3. Load catalog at runtime
4. Test `Addressables.LoadAssetAsync(key)` for each LOD
5. Verify LOD transitions in-game

**Requires**: Unity 2021.3.45f2 + Addressables 1.21.18

### 7. Documentation Updates
**Effort**: Low
**Files to update**:
- [ ] CLI reference with real asset examples
- [ ] Troubleshooting guide (timeout handling, invalid files)
- [ ] Performance expectations (timing per stage)
- [ ] Unity integration guide (prefabs, addressables, LOD groups)

---

## Contingency Plans

### If real models unavailable
- Generate synthetic high-poly models procedurally
- Use existing free Unity assets (Unity Asset Store models)
- Create test suite with parameterized polycount values

### If import still hangs
- Implement async import queue with cancellation tokens
- Split import into separate process (IPC)
- Use file-based caching to avoid re-import

### If LOD quality insufficient
- Implement FastQuadricMeshSimplifier integration (v0.9.0)
- Add configurable simplification algorithm per phase
- Add LOD quality preview mode

---

## Success Criteria

Week 4 is complete when:
1. ✓ Real models imported successfully (no hangs, &lt; 1s per model)
2. ✓ All 9 v0.7.0 models processed through full pipeline
3. ✓ Prefab files generated with valid YAML (verifiable by schema)
4. ✓ Addressables catalog generated with unique GUIDs
5. ✓ Definition updates auto-injected correctly
6. ✓ Performance &lt; 5 min for 9 models (target: 2-3 min)
7. ✓ Visual quality verified (LODs, materials, rigging)

---

## Timeline

- **Days 1-2**: Acquire models, debug import timeout
- **Days 3-4**: Performance profiling, optimization
- **Day 5**: Visual QA, documentation
- **Days 6-7**: Addressables testing, contingency mitigation

---

## Deliverables

### Code
- Diagnostic logging in asset pipeline
- Performance timing instrumentation
- Timeout handling

### Documentation
- Week 4 completion report with metrics
- Performance benchmark results
- Troubleshooting guide
- Unity integration guide

### Artifacts
- 9 optimized .prefab files (v0.7.0)
- Addressables catalog + group YAML
- Performance trace (timing per asset/stage)

---

## Open Questions

1. Where are the real Star Wars Clone Wars models?
   - Sketchfab? Steam Workshop? Local backup?
   - Need to acquire or create substitute models

2. What's causing the import timeout?
   - AssimpNet.Importer.Scene property access?
   - YAML deserialization?
   - Needs debugging with diagnostics

3. Should LOD quality targets be configurable per asset?
   - Currently: 60%/30% hardcoded
   - Option: Move to asset_pipeline.yaml phase config

4. Do we need Addressables at v0.7.0 or can we defer to v0.8.0?
   - Current: Full Addressables in v0.8.0
   - Option: Simplify v0.7.0 to prefabs only, defer catalogs

---

**Owner**: Claude Haiku 4.5
**Date**: 2026-03-13
**Next Phase**: Execute Week 4 (real-world testing)

