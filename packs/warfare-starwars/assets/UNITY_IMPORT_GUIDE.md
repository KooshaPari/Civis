# Unity Import Guide - Star Wars Clone Wars Models (v0.7.0 + v0.8.0)

**Date**: 2026-03-13
**Target**: Unity 2021.3.45f2
**Asset Count**: 13 models (70MB)
**Status**: Ready for import

---

## Quick Start

1. **Copy raw models to Unity project**
   ```bash
   # From project root:
   cp -r packs/warfare-starwars/assets/raw/*/model.* Assets/warfare-starwars/models/
   ```

2. **Import FBX/GLB files**
   - Open each model in Assets folder
   - Set FBX Importer settings (see below)
   - Unity auto-converts GLB → FBX on import

3. **Create prefabs with LOD variants**
   - Use VFXPrefabFactory pattern from existing code
   - Set up LOD0/LOD1/LOD2 variants
   - Add to Addressables catalog

4. **Integrate with ContentLoader**
   - Reference prefabs in unit/building definitions
   - Use `vanilla_mapping_override` for visual asset

---

## Model Import Settings

### FBX Import Configuration

For each `.glb` or `.fbx` file:

```
Model Tab:
- Mesh Compression: High
- Optimize Mesh: ✓ checked
- Read/Write Enabled: ✗ unchecked (unless rigged)

Rigging Tab:
- Animation Type: None (for units)
- Optimize Game Objects: ✓ checked
- Avatar Definition: Create From This Model (rigged only)

Materials Tab:
- Import Materials: ✓ checked
- Import Embedded Materials: ✓ checked
- Material Naming: By Original Material
- Material Search: Local, Imported, All
```

### Settings by Asset Type

#### Characters (Units)
- **Models**: Clone Trooper, General Grievous, ARC Trooper, B2 Super Droid, Droideka
- **Settings**:
  - Rigging: None (static models)
  - LOD: Create LOD0 (full), LOD1 (60%), LOD2 (30%)
  - Scale: 1.0 (keep original)
  - Center mesh at origin

#### Vehicles
- **Models**: AT-TE Walker, AAT Tank
- **Settings**:
  - Rigging: None
  - LOD: Create all 3 variants
  - Scale: 1.0
  - Optimize collision mesh if needed

#### Buildings
- **Models**: Jedi Temple
- **Settings**:
  - Rigging: None
  - LOD: Create all 3 (Jedi Temple high-poly)
  - Scale: Adjust to game scale (typically 0.5-2.0x)
  - Create simple collision box for buildable area

---

## Prefab Creation Workflow

### 1. Create Base Prefab

```csharp
// Manual prefab creation in Unity:
// 1. Drag imported FBX → Scene
// 2. Position at (0, 0, 0)
// 3. Adjust scale if needed
// 4. Right-click → Create Prefab
// 5. Save to: Assets/warfare-starwars/prefabs/{asset_id}.prefab
```

### 2. Set Up LOD Group

```csharp
// In Unity Inspector:
// 1. Select prefab root
// 2. Add Component → LOD Group
// 3. Create LOD Renderers:
//    - LOD0: Full geometry (100% verts) - Screen 100-50%
//    - LOD1: 60% geometry - Screen 50-20%
//    - LOD2: 30% geometry - Screen 20-0%
// 4. Set transition time: 0.5 seconds
```

### 3. Configure Materials

```csharp
// If model has materials:
// 1. Check imported materials in Assets/Materials/
// 2. Assign URP Standard Material
// 3. Set color/texture parameters
// 4. Add faction colors:
//    - Republic: Blue tint (#4488FF)
//    - CIS: Orange tint (#FF4400)
```

### 4. Create Prefab Variant per Faction

```
Assets/warfare-starwars/prefabs/
├── sw_clone_trooper_phase2_sketchfab_001/
│   ├── Base.prefab
│   ├── Clone_Trooper_Republic.prefab (variant)
│   ├── Clone_Trooper_Phase2_Alt.prefab
│   └── Clone_Trooper_LOD_Settings.json
├── sw_general_grievous_sketchfab_001/
│   ├── Base.prefab
│   └── General_Grievous_CIS.prefab (variant)
└── ... (rest of units)
```

---

## Addressables Setup

### 1. Create Addressables Groups

```yaml
# In Addressables Groups window:
Create groups:
  - "sw-models-units" (Republic)
  - "sw-models-units-cis" (CIS)
  - "sw-models-vehicles"
  - "sw-models-buildings"
  - "sw-models-heroes"
```

### 2. Add Prefabs to Addressables

```yaml
# For each prefab, set address:
Format: sw-{unit_id}-{faction}-model

Examples:
  Clone_Trooper_Republic.prefab → "sw-clone-trooper-republic"
  General_Grievous_CIS.prefab → "sw-general-grievous"
  AT_TE_Walker.prefab → "sw-at-te-walker"
  Jedi_Temple.prefab → "sw-jedi-temple"

# Group assignments:
  Republic units → sw-models-units
  CIS units → sw-models-units-cis
  Vehicles → sw-models-vehicles
  Buildings → sw-models-buildings
  Heroes → sw-models-heroes
```

### 3. Build Addressables

```bash
# In Unity Editor:
Window → Asset Management → Addressables → Groups
Right-click → Build → New Build → Default Build Script

# This creates:
# - StreamingAssets/aa/StandaloneWindows64/
# - Catalog files
# - Dependency manifest
```

---

## Integration with ContentLoader

### 1. Update Unit Definitions

```yaml
# In packs/warfare-starwars/units/republic_units.yaml:

- id: clone_trooper
  display_name: Clone Trooper
  vanilla_mapping: line_infantry
  # Add visual asset reference:
  visual_asset: sw-clone-trooper-republic  # Addressables address
  scale: 1.0
  offset: [0, 0, 0]  # If model center needs adjustment
```

### 2. Update Building Definitions

```yaml
# In packs/warfare-starwars/buildings/republic_buildings.yaml:

- id: command_center
  display_name: Command Center
  visual_asset: sw-jedi-temple  # Addressables address
  scale: 1.5  # Larger for building
  placement_type: buildable_area
```

### 3. Load Assets in ContentLoader

```csharp
// In src/SDK/ContentLoader.cs:
// Pseudo-code - actual implementation may vary

private async Task LoadVisualAssets()
{
    var unitDef = registry.Get<UnitDefinition>("clone_trooper");
    var asset = await Addressables.LoadAssetAsync<GameObject>(unitDef.VisualAsset);
    visualCache[unitDef.Id] = asset;
}

// On unit spawn:
var unitDef = registry.Get<UnitDefinition>(unitId);
var prefab = await Addressables.LoadAssetAsync<GameObject>(unitDef.VisualAsset);
var instance = Instantiate(prefab, position, rotation);
```

---

## Model-by-Model Import Checklist

### v0.7.0 Critical Assets

- [ ] **General Grievous** (sw_general_grievous_sketchfab_001)
  - Model: GLB, 4.5k poly
  - Type: Hero character (CIS)
  - LOD: 3 variants
  - Faction color: Orange (#FF4400)
  - Action items:
    - [ ] Import FBX
    - [ ] Create LOD variants
    - [ ] Make prefab
    - [ ] Add to Addressables (sw-general-grievous)
    - [ ] Add faction color material
    - [ ] Update unit definition

- [ ] **Clone Trooper Phase II** (sw_clone_trooper_phase2_sketchfab_001)
  - Model: GLB, 35.6k poly
  - Type: Infantry character (Republic)
  - LOD: 3 variants
  - Faction color: Blue (#4488FF)
  - Action items:
    - [ ] Import FBX
    - [ ] Create LOD variants
    - [ ] Make prefab
    - [ ] Add to Addressables (sw-clone-trooper-republic)
    - [ ] Add faction color material
    - [ ] Replace helmet-only placeholder

- [ ] **AT-TE Walker** (sw_at_te_walker_sketchfab_001)
  - Model: GLB, 61k poly (with animation skeleton)
  - Type: Vehicle (Republic)
  - LOD: 3 variants
  - Action items:
    - [ ] Import FBX
    - [ ] Create LOD variants
    - [ ] Make prefab
    - [ ] Add to Addressables (sw-at-te-walker)
    - [ ] FIX MAPPING: Replace V-19 Torrent model reference
    - [ ] Update building definition

- [ ] **Jedi Temple** (sw_jedi_temple_sketchfab_001)
  - Model: GLB, 106.5k poly
  - Type: Building (Republic)
  - LOD: 3 variants
  - FIRST BUILDING VISUAL
  - Action items:
    - [ ] Import FBX
    - [ ] Create LOD variants
    - [ ] Make prefab
    - [ ] Add to Addressables (sw-jedi-temple)
    - [ ] Scale for building size
    - [ ] Add material (untextured, apply flat color)
    - [ ] Update building definition

- [ ] **B2 Super Battle Droid** (sw_b2_super_droid_sketchfab_001)
  - Model: GLB, 49k poly
  - Type: Heavy infantry (CIS)
  - LOD: 3 variants
  - Faction color: Blue (metallic)
  - Action items:
    - [ ] Import FBX
    - [ ] Create LOD variants
    - [ ] Make prefab
    - [ ] Add to Addressables (sw-b2-super-droid)
    - [ ] Add faction color material
    - [ ] Update unit definition

### v0.8.0 Elite Assets

- [ ] **Clone Trooper Alt** (sw_clone_trooper_phase2_alt_sketchfab_001)
  - Model: GLB, 41.5k poly
  - Type: Infantry (Republic) - visual variety
  - LOD: 3 variants
  - Action items: (similar to primary Clone Trooper)

- [ ] **ARC Trooper** (sw_arc_trooper_sketchfab_001)
  - Model: GLB, 29.6k poly
  - Type: Elite infantry (Republic)
  - LOD: 3 variants
  - Action items: (similar to Clone Trooper, elite variant)

- [ ] **Droideka** (sw_droideka_sketchfab_001)
  - Model: GLB, 257k poly (very high detail)
  - Type: Specialized (CIS)
  - LOD: 3 variants (aggressive LOD reduction needed)
  - Action items: (high-detail model requires special LOD handling)

- [ ] **AAT Tank** (sw_aat_walker_sketchfab_001)
  - Model: GLB, 4k poly (very low poly, game-ready)
  - Type: Vehicle (CIS)
  - LOD: 2 variants (less detail needed for low-poly)
  - Action items: (straightforward low-poly model)

---

## Performance Targets

| Model | Poly Count | LOD0 | LOD1 | LOD2 | Target FPS |
|-------|-----------|------|------|------|-----------|
| Clone Trooper | 35.6k | 35k | 21k | 10k | 60+ |
| AT-TE Walker | 61k | 61k | 36k | 18k | 60+ |
| Jedi Temple | 106.5k | 106k | 64k | 32k | 60+ |
| Droideka | 257k | 257k | 154k | 77k | 60+ |
| AAT Tank | 4k | 4k | 2k | 1k | 120+ |

**Goal**: 256 entities on screen @ 60 FPS with LOD active

---

## Testing Checklist

- [ ] Models render in-game without errors
- [ ] LOD switching works smoothly (no pop-in)
- [ ] Faction colors apply correctly
- [ ] Addressables loading works via ContentLoader
- [ ] No texture/material errors
- [ ] Units spawn with correct visual asset
- [ ] Building placement shows correct model
- [ ] 60 FPS maintained with 64 units on screen
- [ ] No memory leaks from asset loading

---

## Troubleshooting

### Model doesn't import
- Check FBX format version compatibility
- Verify material shader (use URP Standard)
- Try reimporting with default settings

### Materials look wrong
- Verify texture paths in FBX import
- Manually assign URP materials
- Check faction color overlay

### LOD pops in/out visibly
- Increase transition time (0.5 → 1.0 seconds)
- Reduce LOD level difference (100% → 60% → 30%)
- Test at different camera angles

### Addressables not found
- Verify address format: `sw-{unit_id}-{faction}-model`
- Rebuild Addressables catalog
- Check group assignment
- Verify scene has Addressables component

---

## Next Phase (v0.9.0)

Once v0.7.0 + v0.8.0 assets are integrated:
- Import remaining 19 building models
- Create more unit visual variants
- Implement building construction animations
- Add particle effects for destroyed models

---

## References

- **Model Mappings**: `packs/warfare-starwars/assets/MODEL_MAPPINGS.yaml`
- **Sketchfab Sources**: `packs/warfare-starwars/assets/SKETCHFAB_MODELS.json`
- **ContentLoader**: `src/SDK/ContentLoader.cs`
- **Unit Definitions**: `packs/warfare-starwars/units/*.yaml`
- **Building Definitions**: `packs/warfare-starwars/buildings/*.yaml`

---

**Status**: Ready for manual Unity import
**Estimated Time**: 8-12 hours for all v0.7.0 + v0.8.0 assets (Jedi Temple is priority #1 - first building visual)
