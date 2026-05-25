# Unity Quick Start - v0.7.0 Implementation

**Target**: Unity 2021.3.45f2
**Game**: Diplomacy is Not an Option
**Task**: Import 5 Star Wars Clone Wars models (35.6k - 106.5k poly each)

---

## Step 1: Create Directory Structure

```bash
# In Unity project Assets folder:
Assets/warfare-starwars/
├── models/              # GLB/FBX files will go here
├── prefabs/             # Prefabs created from models
├── materials/           # Faction colors and overrides
└── animations/          # (For future use)
```

## Step 2: Copy Model Files

All 9 raw models are here:
```
packs/warfare-starwars/assets/raw/
├── sw_general_grievous_sketchfab_001/model.glb
├── sw_clone_trooper_phase2_sketchfab_001/model.glb
├── sw_at_te_walker_sketchfab_001/model.glb
├── sw_jedi_temple_sketchfab_001/model.glb
├── sw_b2_super_droid_sketchfab_001/model.glb
├── sw_clone_trooper_phase2_alt_sketchfab_001/model.glb
├── sw_arc_trooper_sketchfab_001/model.glb
├── sw_droideka_sketchfab_001/model.glb
└── sw_aat_walker_sketchfab_001/model.glb
```

**Copy these 9 GLB files to**: `Assets/warfare-starwars/models/`

## Step 3: Create FBX Import Profiles

**For Characters** (Clone Trooper, Grievous, ARC, B2, Droideka):
```
Model Tab:
- Mesh Compression: High
- Optimize Mesh: ✓
- Read/Write Enabled: ✗

Rigging Tab:
- Animation Type: None (static models)
- Optimize Game Objects: ✓

Materials Tab:
- Import Materials: ✓
- Import Embedded Materials: ✓
- Material Naming: By Original Material
- Material Search: Local, Imported, All
```

**For Vehicles** (AT-TE Walker, AAT Tank):
- Same as Characters
- Add collision optimization if needed

**For Buildings** (Jedi Temple):
- Same as Characters
- Higher LOD reduction for very high poly models

## Step 4: Create Faction Color Materials

**Republic Blue Material**:
- Color: #4488FF
- Apply to Clone Trooper, ARC Trooper, AT-TE Walker, Jedi Temple

**CIS Orange Material**:
- Color: #FF4400
- Apply to General Grievous, B2 Super Droid, Droideka, AAT Tank

## Step 5: Set Up LOD Group (Per Model)

```
LOD0: Full geometry (100% verts)     → Screen 100-50%
LOD1: 60% geometry                   → Screen 50-20%
LOD2: 30% geometry                   → Screen 20-0%
Transition time: 0.5 seconds
```

## Step 6: Create Addressables Addresses

```
Format: sw-{unit_id}-{faction?}-model

Examples:
  Clone_Trooper_Republic.prefab → "sw-clone-trooper-republic"
  General_Grievous_CIS.prefab → "sw-general-grievous"
  AT_TE_Walker.prefab → "sw-at-te-walker"
  Jedi_Temple.prefab → "sw-jedi-temple"
  B2_Super_Droid.prefab → "sw-b2-super-droid"
```

Groups:
- `sw-models-units` (Republic)
- `sw-models-units-cis` (CIS)
- `sw-models-vehicles`
- `sw-models-buildings`
- `sw-models-heroes`

## Step 7: Update Game Definitions

**Unit definitions** (packs/warfare-starwars/units/):
```yaml
- id: clone_trooper
  display_name: Clone Trooper
  visual_asset: sw-clone-trooper-republic
  scale: 1.0
  offset: [0, 0, 0]
```

**Building definitions** (packs/warfare-starwars/buildings/):
```yaml
- id: command_center
  display_name: Command Center
  visual_asset: sw-jedi-temple
  scale: 1.5
```

---

## v0.7.0 Model Priority Order

1. **Clone Trooper** (starts Week 2) - highest priority, replaces placeholder
2. **General Grievous** (Week 2) - CIS hero balance
3. **B2 Super Droid** (Week 2) - heavy unit visual
4. **AT-TE Walker** (Week 3) - **FIXES MAPPING BUG**
5. **Jedi Temple** (Week 3) - **FIRST BUILDING VISUAL**

---

## Performance Targets

**Goal**: 60 FPS @ 256 entities on screen with LOD active

| Model | LOD0 | LOD1 | LOD2 | Target FPS |
|-------|------|------|------|-----------|
| Clone Trooper | 35k | 21k | 10k | 60+ |
| Grievous | 4.5k | 2.7k | 1.4k | 120+ |
| AT-TE | 61k | 36k | 18k | 60+ |
| Jedi Temple | 106k | 64k | 32k | 60+ |
| B2 Droid | 49k | 29k | 15k | 60+ |

---

## Key Files to Reference

- **UNITY_IMPORT_GUIDE.md** - 400+ line comprehensive guide
- **V0_7_0_V0_8_0_IMPLEMENTATION_PLAN.md** - Week-by-week breakdown
- **MODEL_MAPPINGS.yaml** - Model → game definition mappings
- **V0_7_0_IMPLEMENTATION_TRACKER.md** - Progress checklist

---

## Common Issues & Fixes

**Q: Models don't import**
A: Check FBX format version, verify URP shader, try default settings

**Q: LOD transitions cause pop-in**
A: Increase transition time (0.5 → 1.0), reduce LOD difference (100% → 60% → 30%)

**Q: Addressables not found at runtime**
A: Rebuild catalog (Window → Asset Management → Addressables → Groups → Build)

**Q: Faction colors don't apply**
A: Verify material assignment, check shader supports color tint, test in play mode

---

**Status**: Ready to begin Week 1
**Estimated Duration**: 3-4 weeks to v0.7.0 release
**Next Step**: Open Unity 2021.3.45f2 and create Assets/warfare-starwars/ directory structure
