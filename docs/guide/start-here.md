# DINOForge v0.7.0 + v0.8.0 - START HERE

> **Note**: This file was moved from the project root. Related implementation guides are now in `docs/archive/`.

**Current Status**: ✅ All preparation complete, ready for manual Unity implementation
**Last Updated**: 2026-03-13
**Time to v0.7.0 Release**: 3-4 weeks

---

## What Just Happened

You requested to "begin 0.7\0.8 work in full." This work is now 100% complete in preparation phase:

✅ **Downloaded 9 real Star Wars Clone Wars models** (70MB)
✅ **Created comprehensive implementation guides** (1000+ lines)
✅ **Built asset management system** (manifests, mappings, downloader)
✅ **Documented week-by-week implementation plan** (72 hours effort)
✅ **Created progress trackers and quick-start guides**
✅ **All tests passing** (678/678)

The next phase requires manual Unity work — which is now perfectly documented.

---

## What You Need to Know

### The 9 Models You Now Have

**v0.7.0 Critical** (5 models, ~35% coverage):
1. **Clone Trooper Phase II** — Full body Republic infantry (35.6k poly) ⭐ Replaces helmet placeholder
2. **General Grievous** — CIS hero with 4 arms (4.5k poly) ⭐ Matches Clone Trooper hero
3. **B2 Super Droid** — Heavy CIS unit (49k poly)
4. **AT-TE Walker** — Republic 6-legged vehicle (61k poly) ⭐ **FIXES MAPPING BUG**
5. **Jedi Temple** — Republic building (106.5k poly) ⭐ **FIRST BUILDING VISUAL**

**v0.8.0 Elite** (4 models, ~48% coverage):
6. Clone Trooper Alt — Visual variety (41.5k poly)
7. ARC Trooper — Republic elite (29.6k poly)
8. Droideka — CIS rolling shield droid (257k poly)
9. AAT Tank — CIS ultra-optimized tank (4k poly)

All downloaded to: `packs/warfare-starwars/assets/raw/` (70MB)

---

## Documentation Files Created

**READ IN THIS ORDER:**

### 1. **UNITY_QUICK_START.md** (Quick reference)
   - 5 minute read
   - Copy-paste instructions for Week 1
   - FBX profiles, faction colors, Addressables setup

### 2. **V0_7_0_IMPLEMENTATION_TRACKER.md** (Progress checklist)
   - Week-by-week breakdown
   - Task checklist for all 5 models
   - Success criteria

### 3. **UNITY_IMPORT_GUIDE.md** (Detailed reference)
   - 400+ lines of FBX settings
   - LOD creation workflow
   - Material configuration
   - Per-model import checklist

### 4. **V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md** (Week-by-week plan)
   - Detailed 72-hour, 6-8 week breakdown
   - Risk assessment
   - Performance targets
   - Git commit strategy

### 5. **PROJECT_STATUS.md** (Overall status)
   - Current version & roadmap
   - Test coverage (678 passing)
   - Release timeline

### 6. **V0_7_0_V0_8_0_WORK_COMPLETE.md** (Session summary)
   - What was accomplished
   - What's ready now
   - Handoff checklist

---

## How to Start (Week 1, Days 1-7)

### Step 1: Read & Understand (15 minutes)
```
1. Read: UNITY_QUICK_START.md (sections 1-3)
2. Reference: UNITY_IMPORT_GUIDE.md (Quick Start section)
3. Skim: V0_7_0_IMPLEMENTATION_TRACKER.md (Week 1 tasks)
```

### Step 2: Set Up Unity (2-3 hours)
```
1. Open Unity 2021.3.45f2
2. Create Assets/warfare-starwars/ directory structure:
   ├── models/
   ├── prefabs/
   ├── materials/
   └── animations/
3. Copy GLB files from packs/warfare-starwars/assets/raw/
4. Verify auto-import triggers
```

### Step 3: Create FBX Profiles (2-3 hours)
```
1. Create FBX Importer profile for Characters
2. Create FBX Importer profile for Vehicles
3. Create FBX Importer profile for Buildings
4. Test on Clone Trooper
```

### Step 4: Set Up Materials (1-2 hours)
```
1. Create Republic Blue material (#4488FF)
2. Create CIS Orange material (#FF4400)
3. Test faction color assignment
```

### Step 5: LOD Workflow (2-3 hours)
```
1. Implement LOD reduction for Clone Trooper
2. Create LOD0 (full) → LOD1 (60%) → LOD2 (30%)
3. Test LOD switching in scene
4. Document time per model
```

**Week 1 Deliverable**: FBX profiles working, first LOD test successful

---

## Timeline

```
Week 1:  Foundation & Tooling (40 hours)
         FBX profiles, LOD generation, materials

Week 2:  v0.7.0 Units (15 hours)
         Clone Trooper, Grievous, B2 Super Droid

Week 3:  v0.7.0 Vehicles & Buildings (16 hours)
         AT-TE Walker, Jedi Temple
         + Testing & integration (4 hours)

         → v0.7.0 Release (35% coverage, both heroes)

Week 4-5: v0.8.0 Elite Units (10 hours)
         Clone Alt, ARC Trooper

Week 5:  v0.8.0 Specialized (11 hours)
         Droideka, AAT Tank

Week 6-7: Testing & Optimization (20 hours)
         Performance profiling, integration tests

         → v0.8.0 Release (48% coverage)
```

**Total**: 112 hours (14 work days), split 3-4 weeks

---

## Key Success Metrics

### v0.7.0 (3 weeks)
- [ ] 5 models imported without errors
- [ ] 35% asset coverage (9 visible units, 1 building)
- [ ] Both heroes visible in-game (Clone + Grievous)
- [ ] LOD system working (no pop-in)
- [ ] AT-TE mapping fixed
- [ ] 60 FPS @ 16 units on screen

### v0.8.0 (6-7 weeks)
- [ ] 4 additional models integrated
- [ ] 48% asset coverage (13 visible units, 1 building)
- [ ] All core roles visible
- [ ] 60 FPS @ 32 units on screen

---

## The Model Files

All in `packs/warfare-starwars/assets/raw/`:

```
sw_general_grievous_sketchfab_001/model.glb          (4.5k)
sw_clone_trooper_phase2_sketchfab_001/model.glb      (35.6k)
sw_at_te_walker_sketchfab_001/model.glb              (61k)
sw_jedi_temple_sketchfab_001/model.glb               (106.5k)
sw_b2_super_droid_sketchfab_001/model.glb            (49k)
sw_clone_trooper_phase2_alt_sketchfab_001/model.glb  (41.5k)
sw_arc_trooper_sketchfab_001/model.glb               (29.6k)
sw_droideka_sketchfab_001/model.glb                  (257k)
sw_aat_walker_sketchfab_001/model.glb                (4k)
```

Each has an `asset_manifest.json` with metadata.

---

## Critical Notes

⚠️ **AT-TE Walker Mapping Bug**
- Currently mapped to V-19 Torrent (fighter aircraft model)
- Must be replaced with AT-TE Walker model in Week 3
- Fix: Find `at_te_crew` unit definition and update visual_asset reference

⭐ **Jedi Temple = First Building**
- This is the FIRST visual building asset
- Very high poly (106.5k) — requires aggressive LOD
- Test scaling in-game before finalizing
- This alone justifies v0.7.0 release

✅ **Both Heroes Now Visible**
- Clone Trooper replaces helmet placeholder
- General Grievous is matching CIS hero
- Creates visual balance between factions
- Key for gameplay appeal

---

## Common Questions

**Q: What if I get stuck on LOD generation?**
A: See UNITY_IMPORT_GUIDE.md section "Prefab Creation Workflow" → "Set Up LOD Group"

**Q: How do I know polycount is optimized?**
A: Check UNITY_IMPORT_GUIDE.md table "Performance Targets" for expected LOD levels

**Q: Where do faction colors go?**
A: Create materials in Assets/warfare-starwars/materials/ with colors #4488FF (blue) and #FF4400 (orange)

**Q: How do I add to Addressables?**
A: Follow UNITY_IMPORT_GUIDE.md "Addressables Setup" section — groups and naming scheme provided

**Q: How do I update game definitions?**
A: Edit unit/building YAML in packs/warfare-starwars/units/ and packs/warfare-starwars/buildings/, add `visual_asset:` field

---

## Files Reference

> **Note**: Many of these files have been moved to `docs/archive/`.

```
docs/archive/
├── UNITY_QUICK_START.md ..................... Quick start (read first)
├── V0_7_0_IMPLEMENTATION_TRACKER.md ........ Progress tracker
├── V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md ... Week-by-week plan
├── V0_7_0_V0_8_0_WORK_COMPLETE.md ........ Handoff summary

docs/guide/
├── start-here.md (this file)
└── project-status.md ....................... Overall status

packs/warfare-starwars/
├── pack.yaml ................................. Pack manifest
├── asset_pipeline.yaml ..................... Asset pipeline config
├── addressables.yaml ......................... Addressables catalog
└── assets/raw/
    ├── 9 v0.7.0/v0.8.0 models (GLB files)
    └── 4 v0.6.0 existing models
```

---

## Next Actions

**NOW**: Read UNITY_QUICK_START.md (5 minutes)

**TODAY**: Open Unity 2021.3.45f2, create Assets/warfare-starwars/ structure

**THIS WEEK**: Complete Week 1 (FBX profiles, LOD generation, materials)

**NEXT 3 WEEKS**: Weeks 2-3 (v0.7.0 models and testing)

**RELEASE**: v0.7.0 (35% coverage, both heroes, first building)

---

## Support

- **Questions about implementation?** → See UNITY_IMPORT_GUIDE.md
- **Questions about timeline?** → See V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md
- **Questions about progress?** → Use V0_7_0_IMPLEMENTATION_TRACKER.md
- **Questions about project status?** → See PROJECT_STATUS.md

---

**Status**: ✅ Ready to start Week 1
**Build State**: 678 tests passing, clean git history
**No Blockers**: All groundwork complete

**Now go open Unity.** Everything you need is documented. 🚀
