# Milestone M5: Example Content Packs

**Status:** 🔄 IN PROGRESS
**Target Completion:** Q2 2026
**Owner:** DINOForge Agents

---

## Overview

M5 delivers three fully-functional example mod packs demonstrating DINOForge's content system, asset pipeline, and gameplay integration:

1. **warfare-starwars** — Clone Wars theme (CIS + Republic factions)
2. **warfare-modern** — Modern military (West vs Classic Enemy)
3. **warfare-guerrilla** — Asymmetric warfare faction

All packs progress through:
- Content design & sourcing
- Asset acquisition
- Asset normalization & optimization
- ECS integration & validation
- Gameplay testing & balance
- Pack deployment & documentation

---

## Packs In Development

### 1. warfare-starwars: Clone Wars Theme

**Current Phase:** Asset sourcing and import (Phase 2C/2D)
**Progress:** 3/25 assets normalized, comprehensive unit gap analysis complete

#### Pack Details

- **Theme:** Star Wars Episode II-III Clone Wars (Lucasfilm licensing N/A, fan assets only)
- **Factions:**
  - **Republic** (CIS equivalent) — Clone Troopers, clone armor variants
  - **CIS** (Enemy equivalent) — Battle Droids, droid walker variants
- **Coverage:** 14 unit definitions + 9 building definitions (vs vanilla DINO 28 + 15)

#### Phase 1: Design Complete ✅
- Clone Wars faction doctrine architecture defined
- 10 unit archetypes mapped to vanilla DINO unit types
- Building/structure sourcing strategy documented
- Mod pack manifest structure finalized

#### Phase 2: Asset Sourcing & Normalization 🔄 IN PROGRESS

**Phase 2A-B: Initial Asset Acquisition** ✅ COMPLETE
- 10 high-quality Sketchfab models identified
- License verification (free educational use confirmed)
- Initial mesh quality review (polycount estimates)

**Phase 2C: Comprehensive CIS Unit Gap Analysis** ✅ COMPLETE
- Identified all 58 missing CIS units for vanilla-dino parity
- Prioritized by combat role and uniqueness
- Created sourcing manifest: `/packs/warfare-starwars/PHASE_2C_CIS_SOURCING.md`
- 10 Sketchfab search strategies documented
- Model evaluation criteria established

**Phase 2C-B: Asset Pipeline Normalization** 🔄 IN PROGRESS

**Normalized Assets (3/25 complete):**

| Asset | Original Polys | LOD1 (50%) | LOD2 (25%) | Faction Palette | Status |
|-------|----------------|-----------|-----------|-----------------|--------|
| Clone Trooper Phase II | 35.6K | 17.8K | 8.9K | Republic | ✅ Ready for prototype |
| B2 Super Droid | 49.0K | 24.5K | 12.2K | CIS | ✅ Ready for prototype |
| AAT Lego Walker | 1.4K | 706 | 361 | CIS | ✅ Ready for prototype |

**Processing Pipeline** (Blender 4.5 LTS headless):
1. Download from Sketchfab (Sketchfab API)
2. Normalize (vertex cleanup, duplicate removal, center origin)
3. LOD decimation (50% → 25% polycount via Quadric Mesh Simplification)
4. Faction stylization (palette swap: CIS red/gold, Republic blue/white)
5. Export to GLB + generate preview renders

**Artifact locations:**
- Source meshes: `/packs/warfare-starwars/assets/source/`
- Normalized meshes: `/packs/warfare-starwars/assets/normalized/`
- LOD variants: `/packs/warfare-starwars/assets/lod/`
- Preview renders: `/packs/warfare-starwars/assets/previews/`
- Blend project files: `/packs/warfare-starwars/assets/blender-projects/`

**Technical Status Manifest:** `/packs/warfare-starwars/ASSET_STATUS.md`
- Schema: Downloaded → Normalized → Ready-for-Prototype → Imported → Validated → Optimized → Generated

#### Phase 2D: Bulk Asset Download & Import 🔄 NEXT

**Planned workflow:**
```bash
# 1. Download remaining 22 assets from Sketchfab
dotnet run --project src/Tools/PackCompiler -- sync download warfare-starwars --phase 2d

# 2. Batch normalize in Blender (headless pipeline)
./blender-scripts/normalize-batch.sh /packs/warfare-starwars/assets/source/

# 3. Stylize with faction palettes
./blender-scripts/stylize-batch.sh /packs/warfare-starwars/assets/normalized/

# 4. Import into asset registry
dotnet run --project src/Tools/PackCompiler -- assets import warfare-starwars

# 5. Validate all assets
dotnet run --project src/Tools/PackCompiler -- assets validate warfare-starwars
```

**Completion criteria:**
- All 25 assets at LOD1 + LOD2
- Asset pipeline tests passing (>90% success rate)
- Pack manifest updated with `visual_asset` references
- Playable in-game with correct faction colors

#### Phase 3: Integration & Validation 🔄 PLANNED

- Import processed assets into GameClient asset registry
- Link `visual_asset` fields in unit/building definitions to bundle addresses
- Test asset swaps in-game (gameplay integration)
- Balance validation (unit power levels vs vanilla)

#### Known Issues & Workarounds

| Issue | Root Cause | Workaround | Status |
|-------|-----------|-----------|--------|
| Visual asset name mismatches | Bundle naming didn't match definition IDs | Manual audit + rename in definitions | ✅ Fixed |
| Prefab extraction failures | Direct `LoadAsset&lt;Mesh&gt;` returns null for prefab-built bundles | Fallback to `GameObject` + `MeshFilter`/`MeshRenderer` extraction | ✅ Fixed |
| Asset swap timing | Arbitrary 600-frame delay too aggressive | Swap on first frame where entity count &gt; 0 | ✅ Fixed |
| Duplicate asset IDs | ARF Trooper shared `sw-rep-arc-trooper` with ARC Trooper | Assign distinct `sw-rep-arf-trooper` asset ID | ✅ Fixed |

---

### 2. warfare-modern: Modern Military Theme

**Current Phase:** Framework complete, content sourcing
**Progress:** 2/15 packs sourced

#### Pack Details

- **Theme:** Modern military (tanks, helicopters, infantry)
- **Factions:**
  - **West** (Republic equivalent) — NATO-aligned forces
  - **Classic Enemy** (Enemy equivalent, no faction name change) — Insurgent forces
- **Coverage:** 14 unit definitions + 9 building definitions

#### Status

- Doctrine system validated against Warfare domain
- Unit archetype mapping complete
- Modern military asset sources identified (OpenMilitaryMeshes, Sketchfab, community packs)
- License verification in progress

#### Next Steps

1. Source modern military unit models (tanks, helicopters, jeeps, infantry)
2. Create faction skins (NATO camo colors vs insurgent darker tones)
3. Normalize assets via same Blender pipeline as Star Wars
4. Import + validate in-game
5. Balance tuning (modern firepower vs medieval scaling)

---

### 3. warfare-guerrilla: Asymmetric Faction

**Current Phase:** Doctrine design, archetype validation
**Progress:** Concept complete, sourcing planned

#### Pack Details

- **Theme:** Asymmetric/insurgent warfare (IED, light infantry, terror tactics)
- **Faction:** Guerrilla (new faction vs Enemy)
- **Coverage:** 12 unit definitions + 8 building definitions

#### Concept

- Lower-cost units with guerrilla tactics doctrine
- Higher casualty rates but rapid regeneration
- Asymmetric unit roles (snipers, sappers, IED sappers)
- Economic doctrine emphasis on rapid militia recruitment

#### Status

- Doctrine system designed (low-cost, high-attrition-rate modifier)
- Unit archetypes validated with Warfare domain
- Asset sourcing strategy planned (creative commons, game-ready models)

#### Next Steps

1. Source guerrilla unit models (light infantry, technicals, IED trucks)
2. Validate economic balance vs Republic/Enemy
3. Test asymmetric gameplay scenarios
4. Pack integration + deployment

---

## Asset Pipeline Integration

All three packs use the unified **PackCompiler asset pipeline** (see `/docs/adr/ADR-010-asset-intake-pipeline.md`):

### Mandatory Workflow Steps (for all packs)

1. **Define** — Create/update `asset_pipeline.yaml` in pack root
   - Model source paths (GLB/FBX)
   - LOD targets (polycount percentages)
   - Material definitions (faction colors, emission)
   - Addressables keys
   - Definition updates (inject `visual_asset` references)

2. **Download** — `dotnet run -- sync download <pack>`
   - Sketchfab API integration
   - License verification
   - S3 asset caching

3. **Import** — `dotnet run -- assets import <pack>`
   - GLB/FBX → JSON (via AssimpNet)
   - Mesh metadata extraction
   - Material mapping

4. **Validate** — `dotnet run -- assets validate <pack>`
   - Schema compliance (asset_pipeline.yaml)
   - Mesh integrity checks
   - Addressables reference validation

5. **Optimize** — `dotnet run -- assets optimize <pack>`
   - LOD decimation (QuadricMeshSimplification)
   - Texture compression
   - Material baking

6. **Generate** — `dotnet run -- assets generate <pack>`
   - JSON → Unity .prefab (serialized)
   - Addressables catalog generation
   - Preview screenshots

7. **Verify** — `dotnet run -- assets build <pack>`
   - Full pipeline execution
   - Integration test suite
   - Pack validation

8. **Commit** — Git commit all artifacts + updated definitions

### Asset Services (PackCompiler)

Core services in `src/Tools/PackCompiler/Services/`:

| Service | Responsibility | Tests |
|---------|-----------------|-------|
| `AssetImportService` | GLB/FBX → JSON (via AssimpNet) | 4+ |
| `AssetOptimizationService` | Mesh decimation → LOD variants | 4+ |
| `PrefabGenerationService` | JSON → .prefab (serialized) | 4+ |
| `AddressablesService` | YAML → catalog entries | 2+ |
| `DefinitionUpdateService` | Inject visual_asset into YAML | 2+ |

---

## Testing Requirements

All M5 packs MUST include:

- **Unit tests** for asset import/optimize/generate services
- **Integration tests** for full pipeline (download → build)
- **Regression tests** for known assets (v0.6.0 models, v0.7.0 critical)
- **Performance tests** (import &lt; 5s/model, full pipeline &lt; 5min for 25 models)
- **Schema validation tests** (asset_pipeline.yaml)
- **Pack validation** (manifest completeness, definition references, conflict detection)
- **Gameplay tests** (asset swaps work in-game, balance doesn't break)

Test location: `src/Tests/AssetPipelineTests.cs`

---

## Documentation Requirements

M5 agents MUST maintain:

1. **Pack README** — User guide for each pack (installation, features, compatibility)
2. **Asset Manifest** — Sourcing strategy + technical status (asset-by-asset)
3. **Balance Docs** — Unit stat comparisons vs vanilla
4. **Mod Interactions** — Known conflicts with other packs
5. **Changelog** — Version history and updates

---

## Validation Checklist

- [ ] All 3 packs have valid `pack.yaml` manifest
- [ ] All asset_pipeline.yaml files conform to schema
- [ ] All visual_asset IDs in definitions match bundle addresses
- [ ] Asset import pipeline &lt; 5min for all packs combined
- [ ] All assets render correctly in-game with correct faction colors
- [ ] No texture/material corruption
- [ ] Unit balance tests pass (no 0-damage units, no 99999 HP outliers)
- [ ] Asset swap system works for all 25 units
- [ ] Pack dependencies resolved correctly
- [ ] Conflict detection triggers for incompatible balance packs
- [ ] Documentation complete + reviewed

---

## Dependencies

- **Warfare Domain Plugin** (M4) — Unit/faction/doctrine registry ✅
- **ECS Bridge** (completed) — ComponentMap, EntityQueries, AssetSwapSystem ✅
- **Asset Pipeline** (completed) — AssetsTools.NET, Addressables integration ✅
- **PackCompiler** (M3) — validate, build, assets, sync commands ✅

---

## Blockers & Risks

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Sketchfab API rate limits | Slow asset download | Batch download in off-hours, cache locally |
| Blender headless crashes | Asset pipeline halts | Fallback to manual GLB import |
| License ambiguity | Pack can't be shipped | Pre-verify all model licenses before download |
| In-game asset corruption | Unplayable packs | Regression test suite on known assets |
| Balance variance | Gameplay broken | Statistical balance validation (property tests) |

---

## Success Criteria (Definition of Done)

M5 is complete when:

1. ✅ All 3 packs have playable factions (units render in-game with correct visuals)
2. ✅ Asset pipeline processes all 25+ assets without manual intervention
3. ✅ Balance tests confirm no gameplay-breaking outliers
4. ✅ Documentation is complete and reviewed
5. ✅ CI pipeline validates all packs on every commit
6. ✅ Users can load packs via DesktopCompanion UI (M10 prerequisite)

---

## Timeline

| Phase | Target Date | Status | Deliverables |
|-------|-------------|--------|--------------|
| Design | 2026-02-28 | ✅ Done | Faction doctrines, unit archetypes |
| Asset Sourcing | 2026-03-20 | 🔄 60% | 22/25 assets identified |
| Asset Normalization | 2026-04-10 | 🔄 12% | 3/25 normalized, pipeline working |
| Integration | 2026-04-30 | 🔄 0% | All assets in-game, swaps working |
| Balance + Testing | 2026-05-15 | ⏳ Pending | Unit power levels, conflict tests |
| Pack Release | 2026-05-31 | ⏳ Pending | Deployment docs, GitHub release |

---

## See Also

- **Asset Pipeline Details:** `/docs/adr/ADR-010-asset-intake-pipeline.md`
- **Sourcing Manifest:** `/packs/warfare-starwars/PHASE_2C_CIS_SOURCING.md`
- **Asset Status:** `/packs/warfare-starwars/ASSET_STATUS.md`
- **Agent Tooling Plan:** `/docs/plans/PLAN-agent-tooling-evolution.md`
- **Gameplay Testing:** `/docs/specs/SPEC-003-prove-features-skill.md`

---

**Milestone Owner:** DINOForge Agents
**Last Updated:** 2026-03-24
**Next Review:** Weekly (Mondays)
