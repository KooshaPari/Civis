# v0.7.0 Implementation Tracker

**Status**: Asset preparation complete, ready for manual Unity implementation
**Date Started**: 2026-03-13
**Target Release**: 3-4 weeks (by early April 2026)
**Coverage Target**: 35% (9 visible units, 1 building)

---

## Prerequisites ✅

- [x] All 9 models downloaded (70MB)
- [x] MODEL_MAPPINGS.yaml created with game definition bindings
- [x] UNITY_IMPORT_GUIDE.md written (400+ lines with complete workflow)
- [x] V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md with week-by-week breakdown
- [x] All tests passing (678/678)
- [x] Build clean with zero errors

---

## Week 1: Foundation & Tooling (FBX Profiles, LOD, Materials)

**Days 1-2: Unity Workspace Setup**
- [ ] Open Unity 2021.3.45f2
- [ ] Create `Assets/warfare-starwars/` directory structure
- [ ] Create subdirectories: `models/`, `prefabs/`, `materials/`, `animations/`
- [ ] Copy raw GLB/FBX files from `packs/warfare-starwars/assets/raw/` → `Assets/warfare-starwars/models/`
- [ ] Verify imports trigger automatically

**Days 3-4: Import Settings Optimization**
- [ ] Create FBX Importer profile for **characters**
- [ ] Create FBX Importer profile for **vehicles**
- [ ] Create FBX Importer profile for **buildings**
- [ ] Test on Clone Trooper (reference model)
- [ ] Document any special handling needed

**Day 5: LOD Generation**
- [ ] Implement LOD reduction workflow for Clone Trooper
- [ ] Create LOD0 (full) → LOD1 (60%) → LOD2 (30%)
- [ ] Test LOD switching in scene (no pop-in)
- [ ] Establish time budget: &lt; 3ms per level transition

**Days 6-7: Material Setup**
- [ ] Create faction color materials
  - [ ] Republic blue (#4488FF)
  - [ ] CIS orange (#FF4400)
- [ ] Set up URP shader workflow
- [ ] Create material variants for test unit

**Week 1 Deliverable**: FBX profiles working, first LOD test successful, faction colors configured

---

## Week 2: v0.7.0 Unit Models (Clone Trooper, Grievous, B2)

**Clone Trooper Phase II** (sw_clone_trooper_phase2_sketchfab_001 - 35.6k poly)
- [ ] Import FBX → confirm auto-mesh import
- [ ] Create LOD variants (estimated 6 hours)
- [ ] Create prefab: `Clone_Trooper_Republic.prefab`
- [ ] Add faction color overlay (blue tint)
- [ ] Test spawn in game scene
- [ ] Add to Addressables: `sw-clone-trooper-republic`
- [ ] Update unit definition with visual_asset reference
- **Priority**: CRITICAL - replaces helmet-only placeholder

**General Grievous** (sw_general_grievous_sketchfab_001 - 4.5k poly)
- [ ] Import FBX → confirm 4.5k poly is manageable
- [ ] Create LOD variants (estimated 4 hours)
- [ ] Create prefab: `General_Grievous_CIS.prefab`
- [ ] Test hero unit special properties (if any)
- [ ] Add to Addressables: `sw-general-grievous`
- [ ] Update unit definition
- **Priority**: CRITICAL - CIS hero, balances Republic's Jedi Knight

**B2 Super Battle Droid** (sw_b2_super_droid_sketchfab_001 - 49k poly)
- [ ] Import FBX → confirm 49k poly
- [ ] Create LOD variants (estimated 5 hours)
- [ ] Create prefab: `B2_Super_Droid_CIS.prefab`
- [ ] Add faction color (metallic blue)
- [ ] Add to Addressables: `sw-b2-super-droid`
- [ ] Update unit definition
- **Priority**: HIGH - heavy unit role representation

**Week 2 Deliverable**: 3 unit prefabs working, spawnable in-game, 15 hours effort logged

---

## Week 3: v0.7.0 Vehicles & Buildings (AT-TE Walker, Jedi Temple)

**AT-TE Walker** (sw_at_te_walker_sketchfab_001 - 61k poly with animation)
- [ ] Import FBX → confirm 61k poly with walk animation
- [ ] Create LOD variants (estimated 6 hours)
- [ ] Create prefab: `AT_TE_Walker_Republic.prefab`
- [ ] **CRITICAL**: Fix incorrect V-19 Torrent mapping
  - [ ] Find unit_id: `at_te_crew`
  - [ ] Remove vanilla_mapping to `v19_torrent`
  - [ ] Add visual_asset: `sw-at-te-walker`
- [ ] Test in game (should no longer look like aircraft)
- [ ] Add to Addressables: `sw-at-te-walker`
- [ ] Update unit definition
- **Priority**: CRITICAL BUGFIX - fixes wrong model assignment

**Jedi Temple** (sw_jedi_temple_sketchfab_001 - 106.5k poly - **FIRST BUILDING**)
- [ ] Import FBX → confirm 106.5k poly, untextured architecture
- [ ] Create LOD variants (estimated 6 hours - high detail)
- [ ] Create prefab: `Jedi_Temple_Republic.prefab`
- [ ] Scale model for building size (test in scene)
- [ ] Add material (Jedi Temple gold/orange color)
- [ ] Create buildable area collider
- [ ] Test building placement in editor
- [ ] Add to Addressables: `sw-jedi-temple`
- [ ] Update building definition with visual_asset
- **Priority**: CRITICAL - first building visual, 0% → 5% building coverage

**v0.7.0 Testing & Integration**
- [ ] Play test with all 5 models spawned
- [ ] Check for visual glitches, LOD pops, material issues
- [ ] Verify Addressables loading works
- [ ] Benchmark FPS with 5 hero units on screen
- [ ] Document any fixes needed for v0.8.0

**Week 3 Deliverable**: v0.7.0 complete with 35% asset coverage, both heroes visible, first building working

---

## v0.7.0 Success Criteria

- [x] All 5 models downloaded and verified
- [ ] 5 models imported without errors
- [ ] LOD system working (no visible pop-in)
- [ ] 35% asset coverage achieved
- [ ] Both heroes visible in-game (Clone + Grievous)
- [ ] AT-TE mapping fixed (no longer incorrect aircraft reference)
- [ ] Jedi Temple building first visual implementation complete
- [ ] 60 FPS @ 16 units on screen

---

## Model Import Checklist

### v0.7.0 Critical (5 models - 70MB)

| Model | Type | Polycount | Status | Notes |
|-------|------|-----------|--------|-------|
| Clone Trooper Phase II | Infantry | 35.6k | ⏳ Pending | Replaces helmet-only; Republic core unit |
| General Grievous | Hero | 4.5k | ⏳ Pending | CIS hero; distinctive 4-armed cyborg |
| AT-TE Walker | Vehicle | 61k | ⏳ Pending | **FIXES V-19 mapping**; 6-legged transport |
| Jedi Temple | Building | 106.5k | ⏳ Pending | **FIRST BUILDING VISUAL**; untextured architecture |
| B2 Super Droid | Heavy | 49k | ⏳ Pending | CIS elite droid; blue metallic |

### v0.8.0 Elite (4 models - parallel preparation)

| Model | Type | Polycount | Status | Notes |
|-------|------|-----------|--------|-------|
| Clone Trooper Alt | Infantry | 41.5k | 📋 Queued | Visual variety; same stats |
| ARC Trooper | Elite | 29.6k | 📋 Queued | Republic elite armor |
| Droideka | Specialized | 257k | 📋 Queued | Very high detail rolling droid |
| AAT Tank | Vehicle | 4k | 📋 Queued | Ultra-optimized CIS tank |

---

## Reference Documentation

- **UNITY_IMPORT_GUIDE.md**: Complete FBX/LOD/Addressables workflow
- **V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md**: Week-by-week breakdown with effort estimates
- **MODEL_MAPPINGS.yaml**: Game definition → 3D model bindings
- **SKETCHFAB_MODELS.json**: Model metadata and sources

---

## Time Budget

| Phase | Estimated | Actual | Status |
|-------|-----------|--------|--------|
| Asset acquisition | 12 hrs | 12 hrs | ✅ DONE |
| Week 1 (Tooling) | 40 hrs | — | ⏳ Pending |
| Week 2 (Units) | 15 hrs | — | ⏳ Pending |
| Week 3 (Vehicles/Buildings) | 16 hrs | — | ⏳ Pending |
| **v0.7.0 Total** | **31 hrs** | **12 hrs done** | **19 hrs remaining** |

---

## Next Immediate Steps

1. **Open Unity 2021.3.45f2**
2. **Follow UNITY_IMPORT_GUIDE.md Week 1** (Days 1-2)
   - Create Assets/warfare-starwars/ structure
   - Copy GLB files from packs/warfare-starwars/assets/raw/
   - Verify auto-import triggers
3. **Create first FBX Importer profile** (Characters) - start with Clone Trooper test
4. **Generate first LOD variant** - Clone Trooper LOD0/LOD1/LOD2
5. **Set up faction color materials** - blue #4488FF, orange #FF4400

**Estimated completion**: Early April 2026 (3-4 weeks from start)

---

**Last Updated**: 2026-03-13
**Prepared by**: Claude Haiku 4.5
**Status**: Ready for manual Unity implementation
