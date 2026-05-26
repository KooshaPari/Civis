# Star Wars Clone Wars Mod - Agent Assignment Matrix

## Quick Dispatch Reference

### Phase 1: Foundation Assets (Day 1)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-1 | 1 | Download 3 core assets (Clone, B1, AAT) from Sketchfab; validate licensing | 3 assets | 3d | Day 1 | Day 3 | None |

**Agent-1 Checklist:**
- [ ] Search Sketchfab for "clone trooper lowpoly" → download GLB
- [ ] Search for "B1 battle droid" → download GLB
- [ ] Search for "AAT walker" → download GLB
- [ ] Validate IP: Check author licenses, contact if needed
- [ ] Run: `dotnet run --project src/Tools/PackCompiler -- assets validate packs/warfare-starwars`
- [ ] Update `asset_pipeline.yaml` with LOD targets
- [ ] Commit: assetIndex + asset_pipeline.yaml updates

---

### Phase 2: Asset Sourcing (Day 2-3)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-2 | 2A | Source Clone infantry models (Militia, Trooper, Heavy, Sharpshooter, Medic, ARF) | 6 units | 4d | Day 2 | Day 5 | Phase 1 |
| Agent-3 | 2B | Source vehicles + buildings (BARC, AT-TE, V-19, AAT, STAP, DSD1 + 4 buildings) | 9 items | 4d | Day 2 | Day 5 | Phase 1 |

**Agent-2 Checklist (Infantry Sourcing):**
- [ ] Sketchfab search: "clone trooper phase 2 lowpoly" (target: 1.5K-2.5K tris)
- [ ] Sketchfab search: "clone heavy trooper" (variant or custom mod)
- [ ] Sketchfab search: "clone sniper" OR "marksman lowpoly"
- [ ] Sketchfab search: "clone medic" OR "medical trooper"
- [ ] Sketchfab search: "ARF trooper reconnaissance" OR "scout trooper"
- [ ] Validate all GLBs: geometry, rigs, UV layout
- [ ] Create manifest for each: attachment points (weapon, helmet)
- [ ] Commit: asset manifests + sourcing report

**Agent-3 Checklist (Vehicle + Building Sourcing):**
- [ ] Sketchfab: "BARC speeder" (target: 2K-4K tris)
- [ ] Sketchfab: "AT-TE walker" (target: 5K-8K tris)
- [ ] Sketchfab: "V-19 Torrent starfighter" (target: 3K-5K tris)
- [ ] Sketchfab: "AAT armor assault tank" (Phase 1 already has, verify)
- [ ] Sketchfab: "STAP speeder bike" (target: 1K-2K tris)
- [ ] Sketchfab: "DSD1 dwarf spider droid" (target: 2K-3K tris)
- [ ] Sketchfab: "tatooine building" (target: 3K-5K tris) - 2-3 variants
- [ ] Sketchfab: "geonosis structure" OR "droid factory" (target: 4K-6K tris)
- [ ] Validate all: polycount, material slots, foundation alignment
- [ ] Commit: asset manifests + sourcing report

---

### Phase 3A: Clone Infantry LOD (Day 4-7)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-4 | 3A | Import + LOD: Clone Militia, Trooper, Heavy | 3 units | 4d | Day 4 | Day 7 | Phase 2A |
| Agent-5 | 3A | Import + LOD: Clone Sharpshooter, Medic, ARF Trooper | 3 units | 4d | Day 4 | Day 7 | Phase 2A |

**Agent-4 Checklist:**
- [ ] Import Clone Militia GLB → JSON via `AssetImportService.ImportGLB()`
- [ ] Import Clone Trooper GLB → JSON
- [ ] Import Clone Heavy GLB → JSON
- [ ] Validate geometry: rig structure, bone hierarchy
- [ ] Create LOD variants:
  - LOD0: Full detail (2,500 tri example)
  - LOD1: 50% decimation (1,250 tri)
  - LOD2: 25% decimation (625 tri)
- [ ] Run: `dotnet run --project src/Tools/PackCompiler -- assets optimize packs/warfare-starwars --lod-targets 50,25`
- [ ] Validate: No mesh tears, silhouette preserved, attachment points intact
- [ ] Test: `xUnit AssetOptimizationService.CreateLodVariants()`
- [ ] Commit: Processed JSON + LOD variants

**Agent-5 Checklist:** (Same as Agent-4, different units)
- [ ] Import + validate + LOD: Sharpshooter, Medic, ARF Trooper
- [ ] Test each LOD visually (polycount reduction acceptable)
- [ ] Commit: Processed JSON + LOD variants

---

### Phase 3B: Droid Infantry LOD (Day 4-7)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-6 | 3B | Import + LOD: B1 Battle Droid, B1 Squad, B2 Super Droid | 3 units | 4d | Day 4 | Day 7 | Phase 2A |
| Agent-7 | 3B | Import + LOD: Droideka, BX Commando Droid, MagnaGuard | 3 units | 4d | Day 4 | Day 7 | Phase 2A |

**Agent-6 Checklist:**
- [ ] Import B1 GLB → JSON (expect 1.5K-2.2K base tris)
- [ ] Import B1 Squad variant → JSON (similar or grouped model)
- [ ] Import B2 Super Droid GLB → JSON (more armor, ~2.5K tris)
- [ ] Validate: Mechanical rig (no bones needed, check for collision meshes)
- [ ] Create LOD: 50%, 25% decimation
- [ ] **Material validation:** RED channel for faction color (droid team color)
- [ ] Test: `DroidMaterialSlots` color injection
- [ ] Commit: Processed models + LOD variants

**Agent-7 Checklist:**
- [ ] Import Droideka → JSON (complex shield geometry, special handling)
- [ ] Import BX Commando → JSON (elite droid, smoother design)
- [ ] Import MagnaGuard → JSON (melee specialist, electrostaff rig)
- [ ] LOD: Standard 50%, 25% decimation
- [ ] Material validation: Material slots for red faction color
- [ ] Test: Droideka shield rendering post-LOD (critical visual element)
- [ ] Commit: Models + LOD variants

---

### Phase 3C: Vehicle Models LOD (Day 4-7)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-8 | 3C | Import + LOD: BARC Speeder, AT-TE, V-19 Torrent (Republic vehicles) | 3 units | 4d | Day 4 | Day 7 | Phase 2B |

**Agent-8 Checklist:**
- [ ] Import BARC Speeder → JSON (expect 2K-3K tris, fast vehicle)
- [ ] Import AT-TE Walker → JSON (expect 5K-7K tris, large vehicle)
- [ ] Import V-19 Torrent → JSON (expect 3K-4K tris, starfighter)
- [ ] Validate geometry: Turret attachment points critical
- [ ] LOD targets (higher polycount, vehicles are focal):
  - LOD0: Full detail (6,000 tri example for AT-TE)
  - LOD1: 50% (3,000 tri)
  - LOD2: 25% (1,500 tri)
- [ ] **Attachment validation:** Weapon hard-points preserved (turrets, cannons)
- [ ] Test: `AssetOptimizationService.PreserveAttachmentPoints()`
- [ ] Commit: Models + LOD variants

---

### Phase 3C Parallel: CIS Vehicles LOD (Same timing, separate agent)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-9 | 3C | Import + LOD: AAT, STAP, DSD1 Dwarf Spider (CIS vehicles) | 3 units | 4d | Day 4 | Day 7 | Phase 2B |

**Agent-9 Checklist:**
- [ ] Import AAT → JSON (expect 5K-6K tris, main tank)
- [ ] Import STAP → JSON (expect 1K-2K tris, light vehicle)
- [ ] Import DSD1 Dwarf Spider → JSON (expect 2K-3K tris, walker-tank)
- [ ] LOD: Same targets as Agent-8 (vehicle focal point)
- [ ] Attachment: Validate turret positions for AAT cannon
- [ ] Commit: Models + LOD variants

---

### Phase 4: Building Models LOD (Day 4-9)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-10 | 4 | Import + LOD: Republic Command Center, Clone Facility, Weapons Factory, Guard Tower | 4 buildings | 5d | Day 4 | Day 8 | Phase 2B |
| Agent-11 | 4 | Import + LOD: CIS Tactical Center, Droid Factory, Assembly Line, Sentry Turret | 4 buildings | 5d | Day 4 | Day 8 | Phase 2B |

**Agent-10 Checklist (Republic Buildings):**
- [ ] Import rep_command_center → JSON (target: 4K-5K tris)
- [ ] Import rep_clone_facility → JSON (barracks, target: 3K-4K tris)
- [ ] Import rep_weapons_factory → JSON (variant, target: 3K tris)
- [ ] Import rep_guard_tower → JSON (tower, target: 2K tris)
- [ ] Validate: Flat base (Y=0), interior space walkable
- [ ] LOD targets (buildings larger than units):
  - LOD0: Full detail (5,000 tri)
  - LOD1: 50% (2,500 tri)
  - LOD2: 25% (1,250 tri)
- [ ] Material: Validate faction color slots (Republic = white/blue)
- [ ] Test: `AssetOptimizationService.ValidateFoundation()`
- [ ] Commit: Building models + LOD variants

**Agent-11 Checklist (CIS Buildings):** (Same process, CIS faction)
- [ ] Import cis_tactical_center, cis_droid_factory, cis_assembly_line, cis_sentry_turret
- [ ] Material: Validate faction color (CIS = red/gray)
- [ ] Same LOD, validation as Agent-10
- [ ] Commit: Building models + LOD variants

---

### Phase 5: Prefab Generation + Addressables (Day 9-12)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-12 | 5 | Generate prefabs for all 27 units; create Addressables catalog | 27 units | 4d | Day 9 | Day 12 | Phase 3A, 3B, 3C |
| Agent-13 | 5 | Generate prefabs for 10 buildings; faction color materials | 10 buildings | 4d | Day 9 | Day 12 | Phase 4 |

**Agent-12 Checklist (Unit Prefabs):**
- [ ] Run full pipeline: `dotnet run --project src/Tools/PackCompiler -- assets generate packs/warfare-starwars`
- [ ] Verify: 27 units × 3 LOD levels = 81 unit prefab variants generated
- [ ] Service: `PrefabGenerationService.GenerateUnitPrefabs()`
- [ ] Addressables keys: `sw/units/clone_trooper`, `sw/units/b1_droid`, etc.
- [ ] Material instances: Create faction-specific materials
  - Republic: white/blue color injection
  - CIS: red/gray color injection
- [ ] Test: `xUnit PrefabGenerationService.GenerateUnitPrefabs()`
- [ ] Validate: All prefabs serialized, no missing references
- [ ] Commit: Prefabs + Addressables catalog

**Agent-13 Checklist (Building Prefabs):**
- [ ] Generate: 10 buildings × 3 LOD levels = 30 building prefab variants
- [ ] Service: `PrefabGenerationService.GenerateBuildingPrefabs()`
- [ ] Addressables keys: `sw/buildings/command_center`, `sw/buildings/droid_factory`, etc.
- [ ] Collision meshes: Generate simplified (non-walkable) collision for placement
- [ ] Materials: Same faction color system as units
- [ ] Test: `PrefabGenerationService.GenerateBuildingPrefabs()`
- [ ] Validate: All prefabs ready for game integration
- [ ] Commit: Building prefabs + Addressables catalog

---

### Phase 6: YAML Integration + Mapping (Day 13-17)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-14 | 6 | Map 27 units: Add `visual_asset` + `prefab_address` to unit YAML | 27 units | 5d | Day 13 | Day 17 | Phase 5 |
| Agent-15 | 6 | Map 10 buildings: Add `visual_asset` + `prefab_address` to building YAML | 10 buildings | 5d | Day 13 | Day 17 | Phase 5 |

**Agent-14 Checklist (Unit Mapping):**
- [ ] Edit `packs/warfare-starwars/units/republic_units.yaml`:
  ```yaml
  - id: rep_clone_trooper
    visual_asset:
      asset_id: sw_clone_trooper_phase2_sketchfab_001
      prefab_address: sw/units/clone_trooper
      material_faction_slot: clone_team_color
      attachment_points:
        weapon: dc15a_blaster_right_hand
  ```
- [ ] Edit `packs/warfare-starwars/units/cis_units.yaml`:
  ```yaml
  - id: cis_b1_battle_droid
    visual_asset:
      asset_id: sw_b1_droid_sketchfab_001
      prefab_address: sw/units/b1_battle_droid
      material_faction_slot: droid_team_color
  ```
- [ ] Validate: All 27 units have valid prefab_address
- [ ] Cross-check: Ensure vanilla_mapping matches crosswalk
- [ ] Run validation: `dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars`
- [ ] Test: `ContentLoaderTests.TestUnitYamlToGameEntity()`
- [ ] Commit: Updated unit YAML

**Agent-15 Checklist (Building Mapping):**
- [ ] Edit `packs/warfare-starwars/buildings/republic_buildings.yaml`:
  ```yaml
  - id: rep_command_center
    visual_asset:
      asset_id: sw_republic_command_center_model
      prefab_address: sw/buildings/command_center
      material_faction_slot: republic_team_color
      foundation_height: 0
  ```
- [ ] Edit `packs/warfare-starwars/buildings/cis_buildings.yaml`:
  ```yaml
  - id: cis_droid_factory
    visual_asset:
      asset_id: sw_cis_factory_model
      prefab_address: sw/buildings/droid_factory
      material_faction_slot: droid_team_color
  ```
- [ ] Validate: All 10 buildings have valid prefab_address
- [ ] Run full pipeline: `dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars`
- [ ] Test: `ContentLoaderTests.TestBuildingYamlIntegration()`
- [ ] Commit: Updated building YAML

---

### Phase 7: Quality Assurance + Testing (Day 18-21)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-16 | 7 | Asset + LOD validation tests | All 37 | 4d | Day 18 | Day 21 | Phase 6 |
| Agent-17 | 7 | Integration tests (YAML → ECS) | All 37 | 4d | Day 18 | Day 21 | Phase 6 |
| Agent-18 | 7 | In-game validation (optional, manual play-test) | Sampling | 4d | Day 18 | Day 21 | Phase 6 |

**Agent-16 Checklist (Asset Tests):**
- [ ] Run: `dotnet test src/Tests/AssetPipelineTests.cs --verbosity normal`
- [ ] Target: ≥15 new tests for assets (LOD, materials, attachments)
- [ ] Coverage:
  - LOD generation correctness (unit + building)
  - Material slot validation (faction colors)
  - Prefab serialization completeness
  - Addressables catalog integrity
  - Polycount targets met
- [ ] Report: Test results + any regressions
- [ ] Commit: Test updates if fixes needed

**Agent-17 Checklist (Integration Tests):**
- [ ] Run: `dotnet test src/Tests/ContentLoaderTests.cs src/Tests/RegistryTests.cs`
- [ ] Integration scenarios:
  - Load units.yaml → Verify unit registry entries
  - Load buildings.yaml → Verify building registry entries
  - Crosswalk validation (vanilla ↔ themed mappings)
  - StatModifier application (HP/damage multipliers)
  - Asset reference validation (prefab_address found)
- [ ] Expected: ≥80 total tests passing (80 baseline + 15+ new)
- [ ] Report: Coverage gaps + fix recommendations
- [ ] Commit: New integration tests if any added

**Agent-18 Checklist (In-Game Validation) [OPTIONAL]:**
- [ ] Launch DINO with `-dino-dev` flag
- [ ] Load `warfare-starwars` pack (F9 mod menu)
- [ ] Test: Spawn clone_trooper unit in editor/scenario
  - Verify: Model renders (not missing)
  - Verify: Faction color applies (white/blue for Republic)
  - Verify: Stats apply (125 HP, 14 damage for Clone Trooper)
  - Verify: Animation bone structure works
- [ ] Test: Spawn B1 droid
  - Verify: Model renders
  - Verify: Red/gray faction color
  - Verify: Stats apply
- [ ] Test: Spawn building (command_center)
  - Verify: Model renders at correct scale
  - Verify: Texture quality acceptable
  - Verify: No Z-fighting or gaps
- [ ] Performance: Frame time <16ms (60 FPS target)
- [ ] Report: Visual glitches, missing assets, performance issues
- [ ] Commit: Bug reports (if any)

---

### Phase 8: Release Polish + Sign-Off (Day 22-24)

| Agent | Phase | Task | Units/Buildings | Duration | Start | End | Dependencies |
|-------|-------|------|-----------------|----------|-------|-----|--------------|
| Agent-19 | 8 | Update CHANGELOG + README | Docs | 3d | Day 22 | Day 24 | Phase 7 |
| Agent-20 | 8 | Finalize pack.yaml + asset sourcing report | Manifest | 3d | Day 22 | Day 24 | Phase 7 |

**Agent-19 Checklist (Documentation):**
- [ ] Update `/c/Users/koosh/Dino/CHANGELOG.md`:
  ```markdown
  ## [0.2.0] - 2026-03-20
  ### Added
  - Complete Star Wars Clone Wars faction roster (Republic + CIS)
  - 27 unit models with full YAML definitions + prefabs
  - 10 building models with full YAML definitions + prefabs
  - Asset pipeline integration: 37+ Sketchfab assets sourced
  - Material faction color system (Republic white/blue, CIS red/gray)
  - Addressables catalog for dynamic LOD loading
  - 15+ new unit/building asset tests

  ### Technical
  - Asset sourcing: 100% wrapped (0 custom creation)
  - Pipeline phases: 1-8 complete, 22-day execution
  - Test coverage: 95+ tests passing (80 + 15 new)
  - Performance: <50ms per asset load, <16ms frame time
  ```
- [ ] Update `/c/Users/koosh/Dino/README.md` (if not done):
  - Add Star Wars pack to examples
  - Update faction table
- [ ] Update `docs/warfare/factions.md`:
  - Republic faction: 10 units, 10 buildings, stats/descriptions
  - CIS faction: 9 units, 10 buildings, stats/descriptions
- [ ] Create `docs/asset-sourcing/STARWARS_CREDITS.md`:
  - List all Sketchfab authors + links
  - License compliance notes
  - Modification notes (if any)
- [ ] Commit: All doc updates

**Agent-20 Checklist (Manifest + Release):**
- [ ] Verify `packs/warfare-starwars/pack.yaml`:
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
    factions: 2 (galactic-republic, confederacy-independent-systems)
    units: 27
    buildings: 10
    weapons: 12
    doctrines: 8
    waves: 6
  ```
- [ ] Run final validation:
  ```bash
  dotnet run --project src/Tools/PackCompiler -- validate packs/warfare-starwars
  dotnet run --project src/Tools/PackCompiler -- build packs/warfare-starwars
  ```
- [ ] Expected output: "✓ Pack validation complete. 0 errors, 0 warnings. Ready for distribution."
- [ ] Create asset sourcing report:
  - Sketchfab model links (27 items)
  - Author credits + licenses
  - Modification log (if any)
  - Custom creation: 0 models (GOAL MET)
- [ ] Tag release: `git tag v0.2.0-warfare-starwars`
- [ ] Commit: pack.yaml + manifest updates

---

## Summary Table

| Phase | Agent(s) | Task | Items | Duration | Path | Status |
|-------|----------|------|-------|----------|------|--------|
| 1 | 1 | Foundation assets | 3 | 3d | Critical | Ready |
| 2A | 2 | Infantry sourcing | 6 | 4d | Critical | Ready |
| 2B | 3 | Vehicle sourcing | 9 | 4d | Critical | Ready |
| 3A | 4-5 | Clone LOD | 6 | 4d | Critical | Ready |
| 3B | 6-7 | Droid LOD | 6 | 4d | Critical | Ready |
| 3C | 8-9 | Vehicle LOD | 6 | 4d | Critical | Ready |
| 4 | 10-11 | Building LOD | 10 | 5d | Critical | Ready |
| 5 | 12-13 | Prefab gen | 37 | 4d | Critical | Ready |
| 6 | 14-15 | YAML mapping | 37 | 5d | Critical | Ready |
| 7 | 16-18 | QA testing | 37 | 4d | Critical | Ready |
| 8 | 19-20 | Release | Docs | 3d | Critical | Ready |

**Total Agents:** 20 (can be compressed to 17-18 with agent overlap)
**Total Duration:** 22 days critical path
**Total Effort:** ~77 agent-days

---

**Ready for Dispatch**
