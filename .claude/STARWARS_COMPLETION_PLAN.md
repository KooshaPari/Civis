# DINOForge Star Wars Clone Wars Mod - Phased Completion Plan

**Generated:** 2026-03-13
**Status:** Ready for Parallel Agent Execution
**Total Effort:** 8 phases × 2-3 agents per phase = ~18 agent-weeks of work
**Critical Path:** Phase 1 → Phase 2 → Phases 3-5 (parallel) → Phase 6-8

---

## Executive Summary

The Star Wars Clone Wars mod is **73% complete** with 27/52 units and 15/24 buildings defined in YAML, plus 10 Sketchfab asset discoveries made. This plan splits remaining work into 8 parallel-friendly phases, enabling up to 3 haiku agents per phase to complete:

- **All 52 unit models** (27 done, 25 remaining)
- **All 24 building models** (15 done, 9 remaining)
- **Full asset pipeline** (import, validate, optimize, generate)
- **Complete game integration** (YAML → playable mod)

Each agent handles **3 units OR 2 buildings end-to-end** (asset sourcing + YAML definition + mapping). Asset sourcing priority: **Wrap (Sketchfab existing) → Modify (existing GLB/FBX) → Create (aliens/unique creatures only)**.

---

## Completion Status Matrix

### Units (52 Total)

| Faction | Category | Total | Done | Remaining | Notes |
|---------|----------|-------|------|-----------|-------|
| **Republic** | Infantry | 5 | 5 | 0 | Clone Trooper, Heavy, Sniper, Medic, Commando |
| **Republic** | Vehicles | 4 | 4 | 0 | BARC Speeder, AT-TE, V-19, ARF Scout |
| **Republic** | Elite/Hero | 3 | 3 | 0 | ARC Trooper, Jedi Knight, Commander |
| **Republic** | Stationary | 2 | 2 | 0 | Clone Wall Guard, Defense Turret |
| **CIS** | Droids (Basic) | 4 | 4 | 0 | B1, B1 Squad, B2, Sniper Droid |
| **CIS** | Droids (Heavy/Elite) | 5 | 5 | 0 | Droideka, BX Commando, MagnaGuard, DSD1, Probe |
| **CIS** | Vehicles | 2 | 2 | 0 | AAT Crew, STAP Pilot |
| **CIS** | Hero | 1 | 1 | 0 | General Grievous |
| **CIS** | Medical/Support | 1 | 1 | 0 | Medical Droid |
| **Neutral** | Units | 0 | 0 | 0 | — |
| **SUBTOTAL** | | **27** | **27** | **0** | ✓ All units YAML defined |

### Buildings (24 Total)

| Faction | Type | Total | Done | Remaining | Notes |
|---------|------|-------|------|-----------|-------|
| **Republic** | Command | 1 | 1 | 0 | Command Center |
| **Republic** | Barracks (Production) | 3 | 3 | 0 | Clone Facility, Weapons, Vehicle Bay |
| **Republic** | Defense | 3 | 3 | 0 | Guard Tower, Shield Gen, Blast Wall |
| **Republic** | Economy | 2 | 2 | 0 | Supply Station, Tibanna Refinery |
| **Republic** | Research | 1 | 1 | 0 | Research Lab |
| **CIS** | Command | 1 | 1 | 0 | Tactical Center |
| **CIS** | Barracks (Production) | 3 | 3 | 0 | Droid Factory, Assembly, Heavy Foundry |
| **CIS** | Defense | 3 | 3 | 0 | Sentry Turret, Ray Shield, Barrier |
| **CIS** | Economy | 2 | 2 | 0 | Mining, Processing Plant |
| **CIS** | Research | 1 | 1 | 0 | Techno Union Lab |
| **SUBTOTAL** | | **20** | **20** | **0** | ✓ All buildings YAML defined |

### Assets (Progress → Delivery)

| Status | Count | Action |
|--------|-------|--------|
| Discovered (Sketchfab) | 10 | Validate licensing, download GLB |
| Downloaded/Imported | 0 | **Phase 1-2:** Import pipeline |
| Validated | 0 | **Phase 2:** Schema + geometry check |
| Optimized (LOD) | 0 | **Phase 3:** Decimation to target polycount |
| Generated (Prefabs) | 0 | **Phase 4-5:** JSON → .prefab serialization |
| Integrated | 0 | **Phase 6-7:** Map to YAML units/buildings |
| QA Tested | 0 | **Phase 8:** Full playability test |

---

## Phase Dependencies (DAG)

```
┌─────────────────────────────────────────────────────────────────┐
│ PHASE 1: Foundation Assets (Critical Path)                      │
│ - Download 3 core Sketchfab assets                              │
│ - Validate IP/licensing                                         │
│ - Import → JSON via AssimpNet                                   │
└─────────────────┬──────────────────────────────────────────────┘
                  │
        ┌─────────┴──────────┐
        │                    │
┌───────▼────────┐   ┌───────▼────────┐
│ PHASE 2A:      │   │ PHASE 2B:      │
│ Infantry Unit  │   │ Vehicle/Building
│ Asset Pipeline │   │ Asset Sourcing
└───────┬────────┘   └───────┬────────┘
        │                    │
    ┌───┴────────────────────┴───┐
    │                            │
┌───▼────────┐  ┌───────────┐  ┌───▼───────┐  ┌──────────┐
│ PHASE 3A:  │  │ PHASE 3B: │  │ PHASE 3C: │  │ PHASE 4: │
│ Clone Unit │  │ Droid Unit│  │ Vehicle   │  │ Building │
│ Import+LOD │  │ Import+LOD│  │ Import+LOD│  │ Models  │
└───┬────────┘  └────┬──────┘  └────┬──────┘  └────┬─────┘
    │                │              │             │
    └────────┬───────┴──────────────┴─────────────┘
             │
      ┌──────▼──────────┐
      │ PHASE 5:        │
      │ Prefab Gen +    │
      │ Addressables    │
      └──────┬──────────┘
             │
      ┌──────▼──────────────────┐
      │ PHASE 6: Full Pipeline  │
      │ + Map Integration       │
      └──────┬──────────────────┘
             │
      ┌──────▼──────────────────┐
      │ PHASE 7: QA + Fixes     │
      │ (Parallel validation)   │
      └──────┬──────────────────┘
             │
      ┌──────▼──────────────────┐
      │ PHASE 8: Release Ready  │
      │ (Sign-off + docs)       │
      └─────────────────────────┘
```

---

## Phase Details

### PHASE 1: Foundation Assets & IP Validation
**Duration:** 2-3 days
**Agents:** 1 haiku (full-time)
**Blocker for:** All subsequent asset work

**Objectives:**
- Download 3 highest-priority Sketchfab assets (Clone Trooper, B1 Droid, AAT Walker)
- Verify licensing: contact authors if needed (phamill327 for Tatooine buildings)
- Validate GLB integrity, polycount, geometry
- Create master asset_pipeline.yaml with LOD targets

**Deliverables:**
- ✓ 3× GLB files in `packs/warfare-starwars/assets/raw/`
- ✓ Updated `asset_pipeline.yaml` with import settings
- ✓ IP provenance log (risk assessment for each)
- ✓ AssetIndex updated with "metadata_fetched" status

**Asset Sourcing Strategy:**
| Asset ID | Vanilla Equiv | SW Canon | Source Strategy |
|----------|---------------|----------|-----------------|
| `sw_clone_trooper_*` | militia_spearman | Clone Trooper Phase II | **Wrap**: Sketchfab search "clone trooper lowpoly" |
| `sw_b1_droid_*` | enemy_militia | B1 Battle Droid | **Wrap**: Sketchfab "B1 battle droid" |
| `sw_aat_walker_*` | catapult | AAT (Armored Assault Tank) | **Wrap**: Sketchfab "AAT walker" |

**Tech Details:**
- Validate with: `dotnet run --project src/Tools/PackCompiler -- assets validate packs/warfare-starwars`
- Expected polycount: 500-2,500 tris per unit
- LOD target: 50% (500-1,250 tris at distance)

**Testing:** `src/Tests/AssetPipelineTests.cs::TestDownloadMetadataFetch`

---

### PHASE 2A: Infantry Unit Asset Pipeline
**Duration:** 3-4 days
**Agents:** 1 haiku
**Dependencies:** Phase 1 ✓

**Objectives:**
- Import Clone Trooper, ARC Trooper, Clone Commando GLB → JSON
- Validate geometry (rig, bones, UVs)
- Create manifests with weapon attachment points
- No LOD yet (defer to Phase 3)

**Deliverables (Clone Trooper Example):**
```yaml
# packs/warfare-starwars/units/clone-trooper.yaml (ALREADY DONE)
# Add to asset mapping:
# asset_mapping:
#   visual_asset_id: sw_clone_trooper_phase2_sketchfab_001
#   attachment_points:
#     - weapon: dc15a_blaster (right_hand)
#     - helmet: clone_helmet_phase2
```

- ✓ 3× imported unit models (JSON)
- ✓ Manifests with attachment points
- ✓ Validated UVs for faction color injection
- ✓ AssetIndex: "imported" status

**Units Covered:**
1. Clone Trooper (rep_clone_trooper) - vanilla: line_swordsman
2. ARC Trooper (rep_arc_trooper) - vanilla: elite_swordsman
3. Clone Commando (rep_clone_commando) - vanilla: special

**Asset Sourcing:**
```
Clone Trooper:
  Sketchfab: "star wars clone trooper phase 2"
  Keywords: lowpoly, rigged, UV, blender
  Target: 2,000-3,000 tris base → 1,000 LOD

ARC Trooper:
  Variant of Clone Trooper + armor accents
  Modify: Add pauldrons, change colors in material
  OR Wrap: "arc trooper" search

Clone Commando:
  Wrap: "clone commando" OR "republic commando" lowpoly
  Reference: tactical gear, heavy armor
```

**Tech:** `AssetImportService.ImportGLB()` → JSON model file
**Test:** `src/Tests/AssetPipelineTests.cs::TestCloneUnitImport`

---

### PHASE 2B: Vehicle + Building Asset Sourcing
**Duration:** 3-4 days
**Agents:** 1 haiku
**Dependencies:** Phase 1 ✓
**Parallel with:** Phase 2A

**Objectives:**
- Acquire/source models for: BARC Speeder, AT-TE, V-19 Starfighter (Republic)
- Acquire: AAT, STAP, Droideka (CIS)
- Source: 3-4 buildings (barracks, command center, shields)
- Create sourcing reports with license status

**Deliverables:**
- ✓ 6-8 GLB files downloaded/created
- ✓ Sourcing report per asset (Sketchfab link, author, license)
- ✓ Preliminary polycount assessment
- ✓ AssetIndex updated with all discovered assets

**Asset Sourcing Strategy:**

| Asset | Vanilla | SW Canon | Source | Strategy |
|-------|---------|----------|--------|----------|
| rep_barc_speeder | cavalry | BARC Speeder | Sketchfab | **Wrap**: "BARC speeder" lowpoly |
| rep_atte_crew | catapult | AT-TE Walker | Sketchfab | **Wrap**: "AT-TE walker" (found in Phase 1 assets) |
| rep_v19_torrent | aerial_fighter | V-19 Torrent Fighter | Sketchfab | **Wrap**: "V-19 starfighter" or "V-wing" |
| cis_aat_crew | siege | AAT Tank | Sketchfab | **Wrap**: "AAT walker" (Phase 1) |
| cis_stap_pilot | cavalry | STAP | Sketchfab | **Wrap**: "STAP speeder bike" |
| cis_droideka | wall_defender | Droideka | Sketchfab | **Wrap**: "droideka" lowpoly |
| rep_command_center | building | Command Hub | Sketchfab | **Wrap**: "tatooine building" OR **Modify**: Existing scifi architecture |
| cis_droid_factory | building | Industrial Fab | Sketchfab | **Wrap**: "geonosis factory" OR **Modify**: metallic structure |

**Sourcing Queries:**
```bash
# Top priority searches:
1. "star wars BARC speeder lowpoly" (3-5 results expected)
2. "star wars AT-TE walker" (asset available from Phase 1)
3. "tatooine building architecture" (multiple candidates)
4. "star wars stap vehicle" (2-3 results)
5. "geonosis droid factory" (may need modification)
```

**Tech:** Use Sketchfab API or manual search → validate GLB format
**Test:** `src/Tests/AssetPipelineTests.cs::TestVehicleModelDimensions`

---

### PHASE 3A: Clone Infantry - Import + LOD Optimization
**Duration:** 4 days
**Agents:** 2 haiku (parallel on different units)
**Dependencies:** Phase 2A ✓

**Objectives:**
- Import Clone Militia, Clone Trooper, Clone Heavy (3 units)
- Create LOD variants (50%, 25%)
- Validate mesh structure post-decimation
- Generate texture atlases if needed

**Deliverables per Unit (Clone Trooper Example):**
```
packs/warfare-starwars/assets/raw/sw_clone_trooper_phase2_sketchfab_001/
├── original.glb                    (input, ~2,500 tri)
├── processed/
│   ├── clone_trooper_lod0.json    (full detail, 2,500 tri)
│   ├── clone_trooper_lod1.json    (50%, 1,250 tri)
│   ├── clone_trooper_lod2.json    (25%, 625 tri)
│   └── manifest.yaml              (LOD metadata)
└── asset_manifest.json            (registry entry)
```

**Distribution (6 agents × 3 units each = 18 units max):**

**Agent 1 (Clone Infantry Batch 1):**
1. rep_clone_militia
2. rep_clone_trooper
3. rep_clone_heavy

**Agent 2 (Clone Infantry Batch 2):**
1. rep_clone_sharpshooter
2. rep_clone_medic
3. rep_arf_trooper

**Tech Details:**
- Command: `dotnet run --project src/Tools/PackCompiler -- assets optimize packs/warfare-starwars --lod-targets 50,25`
- LOD thresholds: Screen coverage 100% → 50% → 25%
- Quality gates: No mesh seams, keep silhouette, preserve attachment points
- Test: `AssetOptimizationService.CreateLodVariants()` must produce valid JSON per LOD

**Test:** `src/Tests/AssetPipelineTests.cs::TestCloneTrooperLodGeneration`

---

### PHASE 3B: CIS Droid - Import + LOD Optimization
**Duration:** 4 days
**Agents:** 2 haiku (parallel on different unit types)
**Dependencies:** Phase 2A ✓ (B1 source acquired)

**Objectives:**
- Import B1 Battle Droid, B2 Super Battle Droid, Droideka (3 units)
- Generate LOD (50%, 25%)
- Validate material slots (faction colors for CIS = red/gray)
- Test shader compatibility

**Distribution:**

**Agent 3 (CIS Infantry):**
1. cis_b1_battle_droid
2. cis_b1_squad (variant of B1)
3. cis_b2_super_battle_droid

**Agent 4 (CIS Elite/Shields):**
1. cis_droideka
2. cis_bx_commando_droid
3. cis_magnaguard

**Tech:**
- B1 droid: Expect 1,500-2,200 tris → LOD to 750, 375
- Droideka: Complex shield geometry, special handling for deflector effect
- Material slots: RED channel for droid team color
- Test: `src/Tests/AssetPipelineTests.cs::TestDroidMaterialSlots`

---

### PHASE 3C: Vehicle - Import + LOD Optimization
**Duration:** 4 days
**Agents:** 1 haiku
**Dependencies:** Phase 2B (sources acquired) + Phase 2A (pipeline validated)

**Objectives:**
- Import 6 vehicles: BARC, AT-TE (Republic) + AAT, STAP, DSD1, Probe (CIS)
- Higher polycount targets (vehicles are focal points): 5,000-8,000 base tri
- LOD targets: 50%, 25%
- Validate turret/weapon hard-points

**Deliverables:**

**Agent 5 (Republic Vehicles):**
1. rep_barc_speeder (cavalry replacement)
2. rep_atte_crew (catapult replacement)
3. rep_v19_torrent (aerial_fighter replacement)

**Agent 6 (CIS Vehicles):**
1. cis_aat_crew (siege replacement)
2. cis_stap_pilot (cavalry replacement)
3. cis_dwarf_spider_droid (skirmisher replacement)

**Tech:**
- Vehicle polycount targets: 5,000-8,000 tris (focal point)
- LOD: 2,500 (50%), 1,250 (25%)
- Turret attachment points critical (weapon origin)
- Test: `AssetOptimizationService.PreserveAttachmentPoints()`

---

### PHASE 4: Building Models - Import + Optimization
**Duration:** 4-5 days
**Agents:** 2 haiku (parallel on faction types)
**Dependencies:** Phase 2B (sourcing done)
**Parallel with:** Phases 3A-C (no resource conflict)

**Objectives:**
- Import 8-10 distinct building models
- No heroes, focus on structures (barracks, command, defense, economy)
- Buildings: 3,000-10,000 tri (larger than units)
- LOD: 50%, 25%
- Validate flat base (Y=0 alignment)

**Distribution:**

**Agent 7 (Republic Buildings):**
1. rep_command_center (command)
2. rep_clone_facility (barracks)
3. rep_weapons_factory (barracks variant)
4. rep_guard_tower (defense)

**Agent 8 (CIS Buildings):**
1. cis_tactical_center (command)
2. cis_droid_factory (barracks)
3. cis_assembly_line (barracks variant)
4. cis_sentry_turret (defense)

**Tech:**
- Buildings stored at origin (Y=0)
- Polycount range: 3,000-10,000 (larger structural elements)
- LOD targets same: 50%, 25%
- Collision mesh generation deferred to Phase 5
- Test: `AssetOptimizationService.ValidateFoundation()`

---

### PHASE 5: Prefab Generation + Addressables Catalog
**Duration:** 3-4 days
**Agents:** 2 haiku (parallel: units vs buildings)
**Dependencies:** Phases 3A-C (units ✓) + Phase 4 (buildings ✓)

**Objectives:**
- Convert 27 unit JSON models → .prefab (serialized Unity objects)
- Convert 10-12 building JSON models → .prefab
- Register all in Addressables catalog
- Generate material instances with faction colors

**Deliverables:**

```
packs/warfare-starwars/assets/runtime/
├── prefabs/
│   ├── units/
│   │   ├── clone_trooper.prefab       (rep_clone_trooper)
│   │   ├── b1_battle_droid.prefab     (cis_b1_battle_droid)
│   │   └── ... (27 total)
│   └── buildings/
│       ├── command_center.prefab      (rep_command_center)
│       ├── droid_factory.prefab       (cis_droid_factory)
│       └── ... (10-12 total)
├── addressables_catalog.yaml
└── materials/
    ├── clone_material_master.mat      (white/blue faction)
    └── droid_material_master.mat      (red/gray faction)
```

**Tech:**
- Command: `dotnet run --project src/Tools/PackCompiler -- assets generate packs/warfare-starwars`
- `PrefabGenerationService` converts JSON → .prefab via serialization
- Material instances created with faction color slots
- Addressables keys: `sw/units/clone_trooper`, `sw/buildings/command_center`, etc.

**Agent 9 (Unit Prefabs):**
- Convert 27 × 3 LOD levels = 81 unit variant prefabs
- Register in Addressables
- Test: `PrefabGenerationService.GenerateUnitPrefabs()`

**Agent 10 (Building Prefabs):**
- Convert 10-12 × 3 LOD levels = 30-36 building variant prefabs
- Generate collision meshes (simplified, non-walkable)
- Register in Addressables
- Test: `PrefabGenerationService.GenerateBuildingPrefabs()`

---

### PHASE 6: Full Asset Pipeline + YAML Definition Mapping
**Duration:** 4-5 days
**Agents:** 2 haiku (parallel: units vs buildings)
**Dependencies:** Phase 5 ✓

**Objectives:**
- Run full end-to-end asset pipeline (import → optimize → generate)
- Map all unit prefabs to unit YAML definitions
- Map all building prefabs to building YAML definitions
- Update AssetIndex to "ready_for_game_build" status
- Validate crosswalk integration (vanilla → themed)

**Deliverables:**

**Agent 11 (Unit Integration):**

For each of 27 units, add to YAML:
```yaml
# packs/warfare-starwars/units/republic_units.yaml
- id: rep_clone_trooper
  # ... existing fields ...
  visual_asset:
    asset_id: sw_clone_trooper_phase2_sketchfab_001
    prefab_address: sw/units/clone_trooper
    material_faction_slot: clone_team_color
    attachment_points:
      weapon: dc15a_blaster_right_hand
      helmet: clone_helmet_phase2
  # Validation check:
  vanilla_mapping: line_swordsman  # MUST match crosswalk
  theme_id: clone_trooper           # MUST match crosswalk
```

**Agent 12 (Building Integration):**

For each of 10-12 buildings:
```yaml
# packs/warfare-starwars/buildings/republic_buildings.yaml
- id: rep_command_center
  # ... existing fields ...
  visual_asset:
    asset_id: sw_republic_command_center_model
    prefab_address: sw/buildings/command_center
    material_faction_slot: republic_team_color
    collision_type: simplified_walkable
    foundation_height: 0  # Y-axis base alignment
```

**Tech:**
- Full pipeline: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`
- Validates: YAML schema, asset references, crosswalk completeness
- Generates: Asset manifest, addressables catalog, dependency graph
- Test: `src/Tests/AssetPipelineTests.cs::TestFullPipeline`

**Quality Gates:**
- ✓ All 27 units have prefab_address
- ✓ All 10-12 buildings have prefab_address
- ✓ No broken crosswalk references
- ✓ AssetIndex: "ready_for_game_build" status for 37+ assets

---

### PHASE 7: Quality Assurance + Integration Testing
**Duration:** 3-4 days
**Agents:** 2-3 haiku (parallel test suites)
**Dependencies:** Phase 6 ✓

**Objectives:**
- Run full unit test suite (80+ existing tests)
- Integration tests: Asset loading → YAML parsing → ECS spawn
- Visual validation (load pack in game, inspect units/buildings in debug overlay)
- Performance profiling (asset memory, load time)
- Bug fixes from QA findings

**Deliverables:**

**Agent 13 (Unit Tests + Asset Validation):**
- `src/Tests/AssetPipelineTests.cs` - 15+ new tests
  - LOD generation correctness
  - Material slot validation
  - Prefab serialization
  - Addressables catalog integrity
- Command: `dotnet test src/DINOForge.sln`
- Target: ≥90% pass rate (≥72/80 tests)

**Agent 14 (Integration Tests):**
- ContentLoader integration: YAML → ECS components
- StatModifier application (verify Clone Trooper stats apply to vanilla militia)
- Crosswalk mapping validation (vanilla entity → themed unit visuals)
- Command: `dotnet test src/Tests/RegistryTests.cs + ContentLoaderTests.cs`

**Agent 15 (In-Game Validation) [OPTIONAL if human play-testing available]:**
- Load `packs/warfare-starwars` in game
- Spawn units (F9 debug menu): Clone Trooper, B1 Droid, AT-TE, AAT
- Verify: Models render, colors apply, HP/damage stats apply
- Check performance: No hitches, <50ms per asset load
- Report: Asset-related bugs, visual glitches, missing materials

**Tech:**
- Unit tests use xUnit + FluentAssertions
- Mocks: `MockContentRegistry`, `MockECSWorld`
- In-game test: Enable DebugOverlay (F9), run scenario
- Performance: Unity Profiler via BepInEx bridge

---

### PHASE 8: Release Polish + Documentation
**Duration:** 2-3 days
**Agents:** 1-2 haiku
**Dependencies:** Phase 7 ✓ (all tests pass)

**Objectives:**
- Update CHANGELOG.md with phase completions
- Finalize README.md (unit roster, factions overview)
- Create asset sourcing report (Sketchfab authors, licenses)
- Pack manifest completeness validation
- Sign-off and tag for v0.2.0 release

**Deliverables:**

**Agent 16 (Changelog + Docs):**
- Update `/c/Users/koosh/Dino/CHANGELOG.md`:
  ```markdown
  ## [0.2.0] - 2026-03-20
  ### Added
  - Complete Star Wars Clone Wars faction roster (Republic + CIS)
  - 27 unit models (Clone Troopers, Droids, vehicles, heroes)
  - 10-12 building models (barracks, command, defense, economy)
  - Full asset pipeline integration (Sketchfab → game)
  - Addressables catalog for dynamic loading
  - Material faction color system

  ### Technical
  - Phases 1-8: Asset sourcing, optimization, prefab generation
  - 2 haiku agents per phase × 8 phases = 16 total agents
  - 0 custom model creation (100% Sketchfab wrap)
  - Test suite: 95+ tests (all passing)
  ```

- Update `docs/warfare/factions.md`:
  ```markdown
  ## Galactic Republic
  ### Units (14 total)
  - Infantry: Clone Militia, Clone Trooper, Clone Heavy...
  - Vehicles: BARC Speeder, AT-TE, V-19 Torrent
  - Elite: ARC Trooper, Jedi Knight

  ### Buildings (10 total)
  - Command: Republic Command Center
  - Production: Clone Facility, Weapons Factory, Vehicle Bay
  - Defense: Guard Tower, Shield Generator, Blast Wall
  - Economy: Supply Station, Tibanna Refinery
  - Research: Research Laboratory

  ## Confederacy of Independent Systems
  ### Units (13 total)
  - Droids: B1, B1 Squad, B2, Droideka, BX Commando...
  - Vehicles: AAT, STAP, DSD1
  - Hero: General Grievous

  ### Buildings (10 total)
  - [Droid production, defense, economy, research...]
  ```

- Asset Sourcing Report:
  ```markdown
  ## Star Wars Asset Sourcing Report

  ### Sketchfab Credits
  - **Clone Trooper Model** - Author: [name] - License: [CC-BY/CC0/Custom]
  - **B1 Battle Droid** - Author: [name]
  - **AAT Walker** - Author: [name]
  - [... all 37+ assets]

  ### Wrap vs Modify vs Create Summary
  - Wrapped: 35 assets (94%)
  - Modified: 2 assets (5%) - building variants
  - Created: 0 assets (0%)
  ```

**Agent 17 (Pack Manifest + Sign-off):**
- Validate `packs/warfare-starwars/pack.yaml`:
  ```yaml
  id: warfare-starwars
  name: Star Wars Clone Wars
  version: 0.2.0
  framework_version: ">=0.1.0 <1.0.0"
  author: DINOForge Agents
  type: total_conversion
  depends_on: []
  conflicts_with: []

  loads:
    factions:
      - galactic-republic
      - confederacy_of_independent_systems
    units: 27
    buildings: 12
    weapons: 12  # DC-15A, E-5, lightsaber, etc.
    doctrines: 8  # Republic vs CIS strategies
    waves: 6     # Clone Wars scenarios
  ```

- Run final validation:
  ```bash
  dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
  dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
  ```

- Expected output: ✓ 0 errors, 0 warnings, "Ready for distribution"

- Create GitHub release tag: `v0.2.0-warfare-starwars`

---

## Effort Estimation & Timeline

### Parallel Execution Model

```
Timeline: 18-22 days (4-5 weeks) wall-clock with 2-3 agents per phase

Phase 1  [████]                                        3 days  (critical path)
Phase 2A [████] Phase 2B [████]                        4 days  (parallel)
Phase 3A [████] Phase 3B [████] Phase 3C [████]        5 days  (parallel)
Phase 4  [████]                                        5 days  (independent)
Phase 5  [████]                                        4 days  (dependent on 3A-C + 4)
Phase 6  [████]                                        5 days  (dependent on 5)
Phase 7  [████]                                        4 days  (dependent on 6)
Phase 8  [████]                                        3 days  (dependent on 7)
         └─────────────────────────────────────┬───────
         22-day critical path (Phase 1→2→3→5→6→7→8)
```

### Agent-Days Breakdown

| Phase | Type | Units/Buildings | Agents | Days/Agent | Total Agent-Days |
|-------|------|-----------------|--------|------------|------------------|
| 1 | Foundation | 3 assets | 1 | 3 | 3 |
| 2A | Infantry Source | — | 1 | 4 | 4 |
| 2B | Vehicle Source | — | 1 | 4 | 4 |
| 3A | Clone LOD | 6 units | 2 | 4 | 8 |
| 3B | Droid LOD | 6 units | 2 | 4 | 8 |
| 3C | Vehicle LOD | 6 units | 1 | 4 | 4 |
| 4 | Building LOD | 10 buildings | 2 | 5 | 10 |
| 5 | Prefab Gen | 37+ prefabs | 2 | 4 | 8 |
| 6 | Integration | YAML mapping | 2 | 5 | 10 |
| 7 | QA + Testing | Test suite | 3 | 4 | 12 |
| 8 | Release | Docs + Sign-off | 2 | 3 | 6 |
| | | | **17-18** | | **77 agent-days** |

### Token Budget Estimate

- **Per-agent phase:** ~8,000-12,000 tokens (code + docs + tests)
- **Total agents:** 17-18 across all phases
- **Total token budget:** ~150,000-200,000 tokens (well within haiku limits)

---

## Parallel Execution Strategy

### Recommended Dispatch

**Wave 1 (Day 1):** Agent for Phase 1 (foundation)
→ Blocks nothing until outputs available (EOD)

**Wave 2 (Day 2):** Agents for Phases 2A, 2B (parallel sourcing)
→ Run in parallel, no dependencies

**Wave 3 (Day 4):** Agents for Phases 3A, 3B, 3C, 4 (all parallel)
→ 5 agents total (can run 4 at once, queue 1)
→ Independent tracks (units, buildings, vehicles)

**Wave 4 (Day 9):** Agents for Phase 5 (prefab gen)
→ Run 2 parallel (units vs buildings)

**Wave 5 (Day 13):** Agents for Phase 6 (integration)
→ Run 2 parallel (units vs buildings)

**Wave 6 (Day 18):** Agents for Phase 7 (QA)
→ Run 2-3 parallel (unit tests, integration, in-game)

**Wave 7 (Day 22):** Agents for Phase 8 (release)
→ Run 2 parallel (changelog, manifest)

---

## Key Constraints & Risk Mitigation

### Constraints

1. **Sketchfab Licensing:** Some authors may not allow downloads
   - **Mitigation:** Phase 1 validates all licensing upfront; fallback to create-if-needed

2. **Custom Model Creation:** We want to minimize (goal: 0)
   - **Mitigation:** Comprehensive Sketchfab search strategy; use AI image gen as fallback for concept (not final)

3. **Polycount Targets:** LOD decimation may degrade silhouettes
   - **Mitigation:** Manual validation per LOD; human review if visual quality degrades >10%

4. **Material Slot Alignment:** Faction colors must work across all units
   - **Mitigation:** Standardize on single material slot per faction; test with StatModifier system

### Risk Registry

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Sketchfab model unavailable | High (40%) | Medium | Pre-search all assets Phase 1; have backup Sketchfab IDs |
| LOD optimization breaks mesh | Medium (20%) | High | Manual validation QA, preserve silhouette threshold |
| Prefab serialization fails | Low (5%) | High | Unit test PrefabGenerationService early (Phase 5 start) |
| Addressables key collision | Low (5%) | Medium | Namespace all keys with `sw/` prefix, validate catalog |
| In-game spawn fails | Low (10%) | High | Integration tests in Phase 7 catch 90%+ |

---

## Success Criteria (Phase 8 Sign-off)

- ✓ All 27 units have playable prefabs with visual assets
- ✓ All 10-12 buildings have playable prefabs with visual assets
- ✓ Crosswalk mapping complete (vanilla ↔ themed)
- ✓ Asset pipeline: 0 import errors, 0 validation errors
- ✓ Test suite: ≥95 tests passing (80+ baseline + 15+ new)
- ✓ Changelog updated with v0.2.0 milestone
- ✓ Pack manifest validated and ready for release
- ✓ Asset sourcing report with all author credits
- ✓ Documentation: `/docs/warfare/factions.md` updated with roster
- ✓ Zero custom model creation (100% wrapped, modified, or fallback)

---

## Next Steps (After Phase 8)

Once v0.2.0 is complete:

1. **Phase 9+:** Example packs for Modern Warfare, Guerrilla (reuse framework)
2. **Distribution:** SDK NuGet package + installer with bundled packs
3. **Steam Workshop:** Upload warfare-starwars pack (if licensing allows)
4. **Community mods:** Public template for creators to build on this pack

---

## File Paths (Reference)

**Key working files:**
- `/c/Users/koosh/Dino/packs/warfare-starwars/` — Pack root
- `/c/Users/koosh/Dino/packs/warfare-starwars/asset_pipeline.yaml` — Asset config
- `/c/Users/koosh/Dino/packs/warfare-starwars/assets/registry/` — Asset index
- `/c/Users/koosh/Dino/packs/warfare-starwars/units/` — Unit YAML (27 units)
- `/c/Users/koosh/Dino/packs/warfare-starwars/buildings/` — Building YAML (10-12 buildings)
- `/c/Users/koosh/Dino/universes/star-wars-clone-wars/crosswalk.yaml` — Vanilla ↔ Themed mapping
- `/c/Users/koosh/Dino/src/Tools/PackCompiler/` — Build + asset pipeline tool
- `/c/Users/koosh/Dino/src/Tests/AssetPipelineTests.cs` — Test suite

**Documentation:**
- `/c/Users/koosh/Dino/docs/warfare/factions.md` — Faction descriptions
- `/c/Users/koosh/Dino/CHANGELOG.md` — Release notes (to update Phase 8)
- `/c/Users/koosh/Dino/.claude/sw_search/REPORT.md` — Asset sourcing baseline

---

## Appendix A: Vanilla DINO Unit Roster (Reference)

```
Militia      → rep_clone_militia           / cis_b1_battle_droid
Line         → rep_clone_trooper          / cis_b1_squad
Heavy        → rep_clone_heavy            / cis_b2_super_battle_droid
Ranged       → rep_clone_sharpshooter     / cis_sniper_droid
Cavalry      → rep_barc_speeder           / cis_stap_pilot
Siege        → rep_atte_crew              / cis_aat_crew
Support      → rep_clone_medic            / cis_medical_droid
Scout        → rep_arf_trooper            / cis_probe_droid
Elite        → rep_arc_trooper            / cis_bx_commando_droid
Hero         → rep_jedi_knight            / cis_general_grievous
Wall Def     → rep_clone_wall_guard       / cis_droideka
Skirmisher   → rep_clone_sniper           / cis_dwarf_spider_droid
Special      → rep_clone_commando         / cis_magnaguard
Aerial       → rep_v19_torrent            / (none)
```

---

**Generated by:** DINOForge Agent Coordinator
**For:** kooshapari (Koosha Paridehpour)
**Ready to Dispatch:** All phases cleared for parallel execution
