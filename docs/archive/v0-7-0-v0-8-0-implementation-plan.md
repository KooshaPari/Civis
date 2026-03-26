# v0.7.0 + v0.8.0 Implementation Plan

**Date**: 2026-03-13
**Phase**: Asset Integration & Visual Implementation
**Target Completion**: 6-8 weeks (parallel work possible)
**Current Status**: All models downloaded (70MB, 13 assets), ready for Unity integration

---

## Overview

**v0.7.0** (3 weeks, 5 models):
- Add heroes to both factions (General Grievous, Jedi Knight placeholder)
- Add core infantry unit (Clone Trooper Phase II)
- Fix vehicle mappings (AT-TE Walker)
- **First building visual** (Jedi Temple)
- Add heavy unit (B2 Super Battle Droid)
- **Target**: 45% asset coverage, both factions have heroes

**v0.8.0** (4 weeks, 4 models):
- Add elite unit variants (ARC Trooper, alternative Clone Trooper)
- Add specialized units (Droideka)
- Add support vehicles (AAT Tank)
- **Target**: 60% asset coverage, all core unit roles visible

**Result**: 48% overall coverage (22 visible assets out of 46 definitions), both factions playable with complete unit rosters

---

## Phase 1: v0.7.0 Critical (Weeks 1-3)

### Week 1: Foundation & Tooling
**Goal**: Set up Unity import pipeline, create first prefab templates

- [ ] **Day 1-2: Unity Workspace Setup**
  - Open Unity 2021.3.45f2
  - Create project folder: `Assets/warfare-starwars/`
  - Create subdirectories: `models/`, `prefabs/`, `materials/`, `animations/`
  - Copy raw GLB/FBX files from `packs/warfare-starwars/assets/raw/` → `Assets/warfare-starwars/models/`
  - Verify imports trigger automatically

- [ ] **Day 3-4: Import Settings Optimization**
  - Create FBX Importer profile for characters
  - Create FBX Importer profile for vehicles
  - Create FBX Importer profile for buildings
  - Test on one model (Clone Trooper)
  - Document any special handling needed

- [ ] **Day 5: LOD Generation**
  - Implement LOD reduction workflow for Clone Trooper
  - Create LOD0 (full) → LOD1 (60%) → LOD2 (30%)
  - Test LOD switching in scene
  - Establish time budget: &lt; 3ms per level transition

- [ ] **Day 6-7: Material Setup**
  - Create faction color materials (Republic blue, CIS orange)
  - Set up URP shader workflow
  - Create material variants for Clone Trooper (Republic/CIS if needed)

### Week 2: v0.7.0 Unit Models (Clone Trooper, Grievous, B2)
**Goal**: Get 3 unit models working with proper visuals

- [ ] **Clone Trooper Phase II** (sw_clone_trooper_phase2_sketchfab_001)
  - Import FBX → confirm auto-mesh import
  - Create LOD variants (estimated 6 hours)
  - Create prefab: `Clone_Trooper_Republic.prefab`
  - Add faction color overlay (blue tint)
  - Test spawn in game scene
  - Add to Addressables: `sw-clone-trooper-republic`
  - Update unit definition with visual_asset reference
  - **Status**: CRITICAL - replaces helmet-only placeholder

- [ ] **General Grievous** (sw_general_grievous_sketchfab_001)
  - Import FBX → 4.5k poly, manageable
  - Create LOD variants (estimated 4 hours)
  - Create prefab: `General_Grievous_CIS.prefab`
  - Test hero unit special properties (if any)
  - Add to Addressables: `sw-general-grievous`
  - Update unit definition
  - **Status**: CRITICAL - CIS hero, balances Republic's Jedi Knight

- [ ] **B2 Super Battle Droid** (sw_b2_super_droid_sketchfab_001)
  - Import FBX → 49k poly
  - Create LOD variants (estimated 5 hours)
  - Create prefab: `B2_Super_Droid_CIS.prefab`
  - Add faction color (metallic blue)
  - Add to Addressables: `sw-b2-super-droid`
  - Update unit definition
  - **Status**: HIGH - heavy unit role representation

**Subtotal Week 2**: ~15 hours model work

### Week 3: v0.7.0 Vehicles & Buildings
**Goal**: Fix vehicle mappings, add first building visual

- [ ] **AT-TE Walker Fix** (sw_at_te_walker_sketchfab_001)
  - Import FBX → 61k poly with walk animation included
  - Create LOD variants (estimated 6 hours)
  - Create prefab: `AT_TE_Walker_Republic.prefab`
  - **CRITICAL**: Replace V-19 Torrent mapping
    - Find unit_id: `at_te_crew`
    - Remove vanilla_mapping to `v19_torrent`
    - Add visual_asset: `sw-at-te-walker`
  - Test in game (should no longer look like aircraft)
  - Add to Addressables: `sw-at-te-walker`
  - Update unit definition
  - **Status**: CRITICAL BUGFIX - fixes wrong model assignment

- [ ] **Jedi Temple Building** (sw_jedi_temple_sketchfab_001)
  - Import FBX → 106.5k poly, FIRST BUILDING
  - Create LOD variants (estimated 6 hours - high detail)
  - Create prefab: `Jedi_Temple_Republic.prefab`
  - Scale model for building size (test in scene)
  - Add material (Jedi Temple gold/orange color)
  - Create buildable area collider
  - Test building placement in editor
  - Add to Addressables: `sw-jedi-temple`
  - Update building definition with visual_asset
  - **Status**: CRITICAL - first building visual, 0% → 5% building coverage

- [ ] **v0.7.0 Testing & Integration**
  - Play test with all 5 models spawned
  - Check for visual glitches, LOD pops, material issues
  - Verify Addressables loading works
  - Benchmark FPS with 5 hero units on screen
  - Document any fixes needed for v0.8.0

**Subtotal Week 3**: ~12 hours model work + 4 hours testing

**v0.7.0 Total Effort**: ~31 hours
**v0.7.0 Output**:
- 5 prefabs ready
- 10 Addressables entries (unit variants if needed)
- 35% coverage (up from 26%)
- Both heroes visible
- First building model (Jedi Temple)
- AT-TE mapping fixed

---

## Phase 2: v0.8.0 Elite Units (Weeks 4-7)

### Week 4: Elite Unit Models (ARC Trooper, Alternative Clone Trooper)
**Goal**: Expand unit variety with elite variants

- [ ] **Clone Trooper Phase II Alt** (sw_clone_trooper_phase2_alt_sketchfab_001)
  - Import FBX → 41.5k poly
  - Create LOD variants (estimated 5 hours)
  - Create prefab: `Clone_Trooper_Phase2_Alt.prefab`
  - Add faction color (blue)
  - Add to Addressables: `sw-clone-trooper-phase2-alt`
  - Update unit definition (or use variant)
  - **Purpose**: Visual variety, same stats as primary Clone Trooper
  - **Status**: ENHANCEMENT - improves visual diversity

- [ ] **Clone ARC Trooper** (sw_arc_trooper_sketchfab_001)
  - Import FBX → 29.6k poly
  - Create LOD variants (estimated 5 hours)
  - Create prefab: `ARC_Trooper_Republic.prefab`
  - Add faction color (blue with accent)
  - Add to Addressables: `sw-arc-trooper`
  - Update unit definition
  - **Status**: HIGH - elite unit, shows progression

**Subtotal Week 4**: ~10 hours

### Week 5: Specialized Units (Droideka, AAT Tank)
**Goal**: Add distinctive CIS units

- [ ] **Droideka** (sw_droideka_sketchfab_001)
  - Import FBX → 257k poly (VERY HIGH DETAIL)
  - Create aggressive LOD variants (estimated 8 hours)
    - LOD0: 257k (full)
    - LOD1: 150k (aggressive reduction for high-poly)
    - LOD2: 75k (simplified)
  - Create prefab: `Droideka_CIS.prefab`
  - Add faction color (blue/metallic)
  - Test performance on older systems
  - Add to Addressables: `sw-droideka`
  - Update unit definition
  - **Status**: MEDIUM - distinctive rolling droid, needs LOD attention

- [ ] **AAT Tank** (sw_aat_walker_sketchfab_001)
  - Import FBX → 4k poly (LOW POLY - game-ready)
  - Create LOD variants (estimated 3 hours - minimal reduction needed)
    - LOD0: 4k (full)
    - LOD1: 2.5k (optional)
    - LOD2: 1.5k (optional)
  - Create prefab: `AAT_Tank_CIS.prefab`
  - Add faction color (metallic tan/brown)
  - Add to Addressables: `sw-aat-tank`
  - Update unit definition
  - **Status**: HIGH - CIS heavy vehicle, low overhead

**Subtotal Week 5**: ~11 hours

### Week 6-7: v0.8.0 Testing & Optimization
**Goal**: Ensure all systems work together, optimize for performance

- [ ] **Integration Testing**
  - Spawn all 4 v0.8.0 models together
  - Verify Addressables loads all 9 (5 v0.7.0 + 4 v0.8.0)
  - Test with 16 units on screen (LOD system active)
  - Check for memory leaks
  - Verify no visual glitches

- [ ] **Performance Tuning**
  - Profile FPS with 32 units (16 Republic, 16 CIS)
  - Adjust LOD screen percentages if needed
  - Verify 60 FPS target maintained
  - Document performance characteristics

- [ ] **Balance Review**
  - Units render correctly with faction colors
  - Size relationships feel right (hero vs infantry vs vehicle vs building)
  - Visual readability at distance (LOD1/LOD2)
  - No Z-fighting or clipping

- [ ] **Final Polish**
  - Fix any outstanding material issues
  - Adjust scale if models feel wrong in-game
  - Add any missing shadows/lighting
  - Document known issues for v0.9.0

**Subtotal Weeks 6-7**: ~12 hours testing + ~8 hours optimization

**v0.8.0 Total Effort**: ~41 hours
**v0.8.0 Output**:
- 4 additional prefabs ready
- 8 Addressables entries (including variants)
- 48% coverage (up from 35%)
- Full elite unit roster visible
- High-poly (Droideka) and low-poly (AAT) both optimized

---

## Phase 3: Parallel Work (Optional, can overlap with above)

### Building Asset Search (v0.9.0 Prep)
- Research remaining 19 building models
- Find Sketchfab sources with CC-BY licenses
- Prioritize: Clone Facility, Droid Factory, Guard Tower, Shield Generator
- Goal: Have 5-10 building models identified by end of v0.8.0

### Documentation Updates
- Update CHANGELOG.md with v0.7.0 entry
- Update COVERAGE_MATRIX with new percentages
- Create v0.8.0 release notes
- Add visual screenshots to documentation

### Performance Profiling Framework
- Create FPS benchmark scene with enemy wave spawner
- Measure frame time by system (render, physics, AI, etc.)
- Document baseline: 60 FPS @ 32 units
- Set up performance regression tests

---

## Deliverables by Phase

### v0.7.0 Deliverables
- ✅ 5 unit/building prefabs (Clone Trooper, Grievous, B2, AT-TE, Jedi Temple)
- ✅ Addressables catalog with 10 entries
- ✅ ContentLoader integration for visual assets
- ✅ Unit/building definitions updated with visual_asset references
- ✅ 35% asset coverage achieved
- ✅ Both factions have heroes
- ✅ First building visual (Jedi Temple)
- ✅ AT-TE mapping fixed (no longer aircraft)

### v0.8.0 Deliverables
- ✅ 4 elite unit prefabs (Clone Trooper Alt, ARC, Droideka, AAT)
- ✅ 8 additional Addressables entries
- ✅ 48% asset coverage achieved
- ✅ All core unit roles visually represented
- ✅ Performance profiling & optimization complete
- ✅ Integration testing finished
- ✅ v0.9.0 building asset plan in place

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| High-poly models cause FPS drop | HIGH | Aggressive LOD reduction, pre-profile in Unity |
| Material import issues | MEDIUM | Use URP Standard shader, test early |
| Addressables catalog bloat | LOW | Group assets by type, use labels |
| Asset scale mismatch | MEDIUM | Establish unit size in game units (1 = 1m), test in scene |
| Missing textures in GLB | MEDIUM | Use default gray material, add faction color overlay |

---

## Success Criteria

**v0.7.0**:
- [ ] 5 models imported without errors
- [ ] LOD system working (no visible pop-in)
- [ ] 35% asset coverage (9 / 26 units visible, 1 / 20 buildings)
- [ ] 60 FPS @ 16 units on screen
- [ ] AT-TE mapping fixed
- [ ] Both heroes visible in-game

**v0.8.0**:
- [ ] 4 additional models integrated
- [ ] 48% asset coverage (13 / 26 units visible, 1 / 20 buildings)
- [ ] 60 FPS @ 32 units on screen
- [ ] No visual glitches or material errors
- [ ] All core unit roles (infantry, heavy, elite, vehicle) represented

---

## Timeline Summary

```
Week 1: Tooling & Setup (LOD creation, materials)
Week 2: v0.7.0 Units (Clone Trooper, Grievous, B2)
Week 3: v0.7.0 Vehicles & Buildings (AT-TE, Jedi Temple)
Week 4: v0.8.0 Infantry (ARC, Alt Clone)
Week 5: v0.8.0 Specialized (Droideka, AAT)
Week 6-7: Testing, optimization, documentation

Total: 72 hours (9 work days at 8 hours/day)
```

---

## Git Commit Strategy

- **After Week 2**: `feat: import v0.7.0 unit models (Clone Trooper, Grievous, B2)`
- **After Week 3**: `feat: complete v0.7.0 (Jedi Temple building, AT-TE fix, 35% coverage)`
- **After Week 5**: `feat: import v0.8.0 elite units (ARC, Droideka, AAT, 48% coverage)`
- **After Week 7**: `feat: complete v0.8.0 with testing & optimization`

Each commit includes:
- Updated CHANGELOG.md
- Updated COVERAGE_MATRIX.txt
- Performance benchmarks
- Integration test results

---

## Next Immediate Steps

1. **Export this plan as GitHub Issue template** (for tracking)
2. **Set up Unity project** (if not already done)
3. **Start Week 1 tooling** (FBX import profiles, LOD generation)
4. **First model import**: Clone Trooper (highest priority)
5. **Weekly progress commits** to maintain momentum

---

**Prepared by**: Claude Haiku 4.5
**Status**: Ready for implementation
**Estimated Completion**: 6-8 weeks (parallel work can compress to 4-5 weeks)
